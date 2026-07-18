use std::{
    collections::VecDeque,
    ffi::OsString,
    io::{self, BufRead, BufReader},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio},
};
use upscale_contract::{FailureKind, OutputFormat, UpscaleEngine};

pub const LOG_TAIL_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub struct EngineInvocation {
    pub engine: UpscaleEngine,
    pub program: PathBuf,
    pub arguments: Vec<OsString>,
}

impl EngineInvocation {
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.arguments);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        command
    }
}

pub trait EngineProcess: Send {
    fn take_stdout(&mut self) -> io::Result<Box<dyn BufRead + Send>>;
    fn take_stderr(&mut self) -> io::Result<Box<dyn BufRead + Send>>;
    fn wait(&mut self) -> io::Result<ExitStatus>;
    fn kill_process_group(&mut self) -> io::Result<()>;
    fn process_group_id(&self) -> i32;
}

pub trait EngineRunner: Send + Sync {
    fn spawn(&self, invocation: &EngineInvocation) -> io::Result<Box<dyn EngineProcess>>;
}

#[derive(Default)]
pub struct CommandRunner;

impl EngineRunner for CommandRunner {
    fn spawn(&self, invocation: &EngineInvocation) -> io::Result<Box<dyn EngineProcess>> {
        let child = invocation.command().spawn()?;
        Ok(Box::new(CommandProcess { child }))
    }
}

struct CommandProcess {
    child: Child,
}

impl EngineProcess for CommandProcess {
    fn take_stdout(&mut self) -> io::Result<Box<dyn BufRead + Send>> {
        let stream: ChildStdout = self
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("upscaler stdout was not piped"))?;
        Ok(Box::new(BufReader::new(stream)))
    }

    fn take_stderr(&mut self) -> io::Result<Box<dyn BufRead + Send>> {
        let stream: ChildStderr = self
            .child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("upscaler stderr was not piped"))?;
        Ok(Box::new(BufReader::new(stream)))
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }

    fn kill_process_group(&mut self) -> io::Result<()> {
        kill_process_group(self.process_group_id(), libc::SIGTERM)
    }

    fn process_group_id(&self) -> i32 {
        self.child.id() as i32
    }
}

pub fn kill_process_group(group_id: i32, signal: i32) -> io::Result<()> {
    let result = unsafe { libc::kill(-group_id, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

pub struct EngineAdapter {
    engine: UpscaleEngine,
}

impl EngineAdapter {
    pub fn new(engine: UpscaleEngine) -> Self {
        Self { engine }
    }

    pub fn build_invocation(
        &self,
        binary: &Path,
        source: &Path,
        temporary: &Path,
        models: &Path,
        model: &str,
        scale: u32,
        format: OutputFormat,
    ) -> EngineInvocation {
        let mut arguments = vec![
            OsString::from("-i"),
            source.as_os_str().to_owned(),
            OsString::from("-o"),
            temporary.as_os_str().to_owned(),
        ];
        if self.engine == UpscaleEngine::Upscayl {
            arguments.extend([OsString::from("-z"), OsString::from("4")]);
        }
        arguments.extend([
            OsString::from("-s"),
            OsString::from(scale.to_string()),
            OsString::from("-m"),
            models.as_os_str().to_owned(),
            OsString::from("-n"),
            OsString::from(model),
            OsString::from("-f"),
            OsString::from(format.extension()),
        ]);
        EngineInvocation {
            engine: self.engine,
            program: binary.to_path_buf(),
            arguments,
        }
    }

    pub fn classify_diagnostics(
        &self,
        status: &ExitStatus,
        stdout: &str,
        stderr: &str,
    ) -> FailureKind {
        let diagnostics = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        if [
            "no vulkan device",
            "vkcreateinstance failed",
            "vkcreatedevice failed",
            "invalid gpu device",
        ]
        .iter()
        .any(|signature| diagnostics.contains(signature))
        {
            FailureKind::EngineIncompatibility
        } else if diagnostics.contains("no space left on device")
            || status.code() == Some(libc::ENOSPC)
        {
            FailureKind::DiskSpace
        } else {
            FailureKind::EngineProcessing
        }
    }

    pub fn owned_companion_paths(&self, temporary: &Path, format: OutputFormat) -> Vec<PathBuf> {
        if self.engine == UpscaleEngine::RealEsrgan && format == OutputFormat::Jpg {
            let mut name = temporary.as_os_str().to_owned();
            name.push(".png");
            vec![PathBuf::from(name)]
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BoundedTail {
    bytes: VecDeque<u8>,
}

impl BoundedTail {
    pub fn push(&mut self, bytes: &[u8]) {
        if bytes.len() >= LOG_TAIL_BYTES {
            self.bytes.clear();
            self.bytes.extend(&bytes[bytes.len() - LOG_TAIL_BYTES..]);
            return;
        }
        let overflow =
            self.bytes.len() + bytes.len() - (self.bytes.len() + bytes.len()).min(LOG_TAIL_BYTES);
        self.bytes.drain(..overflow);
        self.bytes.extend(bytes);
    }

    pub fn to_lossy_string(&self) -> String {
        let bytes = self.bytes.iter().copied().collect::<Vec<_>>();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

pub fn parse_progress(line: &[u8]) -> Option<f32> {
    let text = String::from_utf8_lossy(line);
    let trimmed = text.trim();
    let number = trimmed.strip_suffix('%')?;
    if number.is_empty()
        || number
            .chars()
            .any(|character| !(character.is_ascii_digit() || character == '.'))
    {
        return None;
    }
    let value = number.parse::<f32>().ok()?;
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=100.0).contains(value))
}

pub fn accept_progress(last: &mut Option<f32>, line: &[u8]) -> Option<f32> {
    let value = parse_progress(line).filter(|value| *value < 100.0)?;
    if last.is_some_and(|previous| value < previous) {
        return None;
    }
    *last = Some(value);
    Some(value)
}

pub fn drain_stream(
    mut stream: Box<dyn BufRead + Send>,
    mut on_line: impl FnMut(&[u8]),
) -> io::Result<BoundedTail> {
    let mut tail = BoundedTail::default();
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = stream.read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        tail.push(&line);
        on_line(&line);
    }
    Ok(tail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn adapters_keep_path_arguments_and_engine_scale_rules() {
        let source = Path::new("/tmp/source image.png");
        let output = Path::new("/tmp/output image.png");
        let models = Path::new("/tmp/models dir");
        let upscayl = EngineAdapter::new(UpscaleEngine::Upscayl).build_invocation(
            Path::new("/bin/upscayl"),
            source,
            output,
            models,
            "model",
            3,
            OutputFormat::Png,
        );
        assert!(upscayl.arguments.windows(2).any(|pair| pair == ["-z", "4"]));
        assert!(upscayl.arguments.windows(2).any(|pair| pair == ["-s", "3"]));
        assert!(upscayl.arguments.contains(&source.as_os_str().to_owned()));

        let official = EngineAdapter::new(UpscaleEngine::RealEsrgan).build_invocation(
            Path::new("/bin/official"),
            source,
            output,
            models,
            "model",
            2,
            OutputFormat::Jpg,
        );
        assert!(!official.arguments.iter().any(|arg| arg == "-z"));
        assert!(official
            .arguments
            .windows(2)
            .any(|pair| pair == ["-s", "2"]));
        assert_eq!(
            EngineAdapter::new(UpscaleEngine::RealEsrgan)
                .owned_companion_paths(output, OutputFormat::Jpg),
            vec![PathBuf::from("/tmp/output image.png.png")]
        );
    }

    #[test]
    fn progress_is_strict_finite_and_bounded() {
        assert_eq!(parse_progress(b"12.5%\n"), Some(12.5));
        for invalid in [
            b"progress 12%".as_slice(),
            b"101%",
            b"NaN%",
            b"12%%",
            b"-1%",
        ] {
            assert_eq!(parse_progress(invalid), None);
        }
    }

    #[test]
    fn progress_is_monotonic_and_reserves_100_for_completion() {
        let mut last = None;
        assert_eq!(accept_progress(&mut last, b"10%"), Some(10.0));
        assert_eq!(accept_progress(&mut last, b"9%"), None);
        assert_eq!(accept_progress(&mut last, b"10%"), Some(10.0));
        assert_eq!(accept_progress(&mut last, b"100%"), None);
        assert_eq!(last, Some(10.0));
    }

    #[test]
    fn stream_drain_tolerates_invalid_utf8_and_bounds_tail() {
        let mut bytes = vec![b'x'; LOG_TAIL_BYTES + 10];
        bytes.extend([0xff, b'\n']);
        let tail = drain_stream(Box::new(Cursor::new(bytes)), |_| {}).unwrap();
        assert!(tail.bytes.len() <= LOG_TAIL_BYTES);
        assert!(tail.to_lossy_string().ends_with("�\n"));
    }
}
