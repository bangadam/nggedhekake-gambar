use fs2::FileExt;
use image::{ImageFormat, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{self, BufReader},
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
    thread,
};
use upscale_contract::{DiskSpaceWarning, OutputFormat, StartupStatus, UpscaleEngine};
use uuid::Uuid;

pub const MAX_OUTPUT_DIMENSION: u32 = 16_383;
pub const MAX_VALIDATION_ALLOCATION: u64 = 1024 * 1024 * 1024;
const MIN_SPACE_MARGIN: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Preflight {
    pub run_id: Uuid,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub models: PathBuf,
    pub binary: PathBuf,
    pub source_dimensions: [u32; 2],
    pub output_dimensions: [u32; 2],
    pub required_bytes: u64,
}

#[derive(Debug)]
pub enum PreflightError {
    Source(String),
    SafeLimit {
        message: String,
        dimensions: Option<[u32; 2]>,
    },
    Destination(String),
    MissingResource(String),
    Internal(String),
}

pub fn preflight(
    run_id: &str,
    image_path: &str,
    output_folder: &str,
    model: &str,
    scale: u32,
    engine: UpscaleEngine,
    resources: &Path,
) -> Result<Preflight, PreflightError> {
    let run_id = Uuid::parse_str(run_id)
        .map_err(|_| PreflightError::Internal("Run ID is not a valid UUID".into()))?;
    if !matches!(scale, 2 | 3 | 4) {
        return Err(PreflightError::SafeLimit {
            message: "Scale must be 2, 3, or 4".into(),
            dimensions: None,
        });
    }

    let source = PathBuf::from(image_path);
    let source_format = image::guess_format(
        &fs::read(&source).map_err(|error| PreflightError::Source(error.to_string()))?,
    )
    .map_err(|error| PreflightError::Source(error.to_string()))?;
    if !matches!(
        source_format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return Err(PreflightError::Source(
            "Only PNG, JPEG, and WebP sources are supported".into(),
        ));
    }
    let dimensions = ImageReader::open(&source)
        .and_then(|reader| reader.with_guessed_format())
        .map_err(|error| PreflightError::Source(error.to_string()))?
        .into_dimensions()
        .map_err(|error| PreflightError::Source(error.to_string()))?;
    if dimensions.0 == 0 || dimensions.1 == 0 {
        return Err(PreflightError::Source(
            "Source dimensions must be non-zero".into(),
        ));
    }
    let output_dimensions = checked_output_dimensions(dimensions, scale).map_err(|message| {
        PreflightError::SafeLimit {
            message,
            dimensions: Some([dimensions.0, dimensions.1]),
        }
    })?;
    let output_bytes = rgba_bytes(output_dimensions).ok_or_else(|| PreflightError::SafeLimit {
        message: "Output size overflowed the safe allocation limit".into(),
        dimensions: Some([dimensions.0, dimensions.1]),
    })?;
    if output_bytes > MAX_VALIDATION_ALLOCATION {
        return Err(PreflightError::SafeLimit {
            message: "Output validation would exceed the 1 GiB safe allocation limit".into(),
            dimensions: Some([dimensions.0, dimensions.1]),
        });
    }

    let destination = PathBuf::from(output_folder);
    if !destination.is_dir() {
        return Err(PreflightError::Destination(
            "Destination folder does not exist".into(),
        ));
    }
    prove_writable(&destination, run_id)
        .map_err(|error| PreflightError::Destination(error.to_string()))?;

    let models = resources.join("models");
    if !models.join(format!("{model}.param")).is_file()
        || !models.join(format!("{model}.bin")).is_file()
    {
        return Err(PreflightError::MissingResource(format!(
            "Model is incomplete: {model}"
        )));
    }
    let binary = resources.join(match engine {
        UpscaleEngine::Upscayl => "binaries/upscayl-bin",
        UpscaleEngine::RealEsrgan => "binaries/realesrgan-ncnn-vulkan",
    });
    if !binary.is_file() {
        return Err(PreflightError::MissingResource(
            "Upscaling engine is missing".into(),
        ));
    }

    Ok(Preflight {
        run_id,
        source,
        destination,
        models,
        binary,
        source_dimensions: [dimensions.0, dimensions.1],
        output_dimensions,
        required_bytes: required_space(output_bytes),
    })
}

pub fn paired_models(resources: &Path) -> Result<Vec<String>, String> {
    let models_path = resources.join("models");
    let mut models = fs::read_dir(&models_path)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()?.to_str()? == "param")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))
                .flatten()
        })
        .filter(|model| models_path.join(format!("{model}.bin")).is_file())
        .collect::<Vec<_>>();
    models.sort();
    Ok(models)
}

pub fn checked_output_dimensions(dimensions: (u32, u32), scale: u32) -> Result<[u32; 2], String> {
    let width = dimensions
        .0
        .checked_mul(scale)
        .ok_or_else(|| "Output width overflowed".to_string())?;
    let height = dimensions
        .1
        .checked_mul(scale)
        .ok_or_else(|| "Output height overflowed".to_string())?;
    if width > MAX_OUTPUT_DIMENSION || height > MAX_OUTPUT_DIMENSION {
        return Err(format!(
            "Output dimensions {width}×{height} exceed the {MAX_OUTPUT_DIMENSION}px safe limit"
        ));
    }
    Ok([width, height])
}

pub fn rgba_bytes(dimensions: [u32; 2]) -> Option<u64> {
    u64::from(dimensions[0])
        .checked_mul(u64::from(dimensions[1]))?
        .checked_mul(4)
}

pub fn required_space(output_bytes: u64) -> u64 {
    output_bytes.saturating_add(MIN_SPACE_MARGIN.max(output_bytes / 10))
}

pub fn disk_warning(preflight: &Preflight) -> Option<DiskSpaceWarning> {
    disk_warning_for_available(
        preflight.required_bytes,
        fs2::available_space(&preflight.destination).ok(),
    )
}

fn disk_warning_for_available(
    required_bytes: u64,
    available_bytes: Option<u64>,
) -> Option<DiskSpaceWarning> {
    let available_bytes = available_bytes?;
    (available_bytes < required_bytes).then(|| DiskSpaceWarning {
        message: "This destination may not have enough free space.".into(),
        required_bytes,
        available_bytes,
    })
}

fn prove_writable(destination: &Path, run_id: Uuid) -> io::Result<()> {
    let probe = destination.join(format!(".nggedhekake-gambar.{run_id}.write-probe"));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)?;
    file.sync_all()?;
    drop(file);
    fs::remove_file(probe)
}

pub fn temporary_path(destination: &Path, run_id: Uuid, format: OutputFormat) -> PathBuf {
    destination.join(format!(
        ".nggedhekake-gambar.{run_id}.tmp.{}",
        format.extension()
    ))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalRecord {
    pub run_id: Uuid,
    pub engine: UpscaleEngine,
    pub owned_paths: Vec<PathBuf>,
}

pub struct OwnedJournal {
    file: File,
    path: PathBuf,
    pub record: JournalRecord,
}

impl OwnedJournal {
    pub fn create(
        runs_dir: &Path,
        run_id: Uuid,
        engine: UpscaleEngine,
        owned_paths: Vec<PathBuf>,
    ) -> io::Result<Self> {
        fs::create_dir_all(runs_dir)?;
        let path = runs_dir.join(format!("{run_id}.json"));
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        let mut file = options.open(&path)?;
        file.lock_exclusive()?;
        let record = JournalRecord {
            run_id,
            engine,
            owned_paths,
        };
        serde_json::to_writer(&mut file, &record).map_err(io::Error::other)?;
        file.sync_all()?;
        Ok(Self { file, path, record })
    }

    pub fn set_inherited(&self, inherited: bool) -> io::Result<()> {
        let fd = self.file.as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags == -1 {
            return Err(io::Error::last_os_error());
        }
        let updated = if inherited {
            flags & !libc::FD_CLOEXEC
        } else {
            flags | libc::FD_CLOEXEC
        };
        if unsafe { libc::fcntl(fd, libc::F_SETFD, updated) } == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn cleanup(self) -> io::Result<()> {
        let mut first_error = None;
        for owned in &self.record.owned_paths {
            if let Err(error) = remove_owned(owned) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        let path = self.path.clone();
        drop(self);
        fs::remove_file(path)
    }
}

fn remove_owned(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn cleanup_startup(runs_dir: &Path) -> StartupStatus {
    let mut status = StartupStatus::default();
    let entries = match fs::read_dir(runs_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return status,
        Err(_) => {
            status
                .cleanup_failures
                .push("an app-owned destination (journal directory unavailable)".into());
            return status;
        }
    };
    for entry in entries.filter_map(Result::ok) {
        let journal_path = entry.path();
        if journal_path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open(&journal_path)
        {
            Ok(file) => match file.try_lock_exclusive() {
                Ok(()) => {
                    if cleanup_journal_file(file, &journal_path).is_err() {
                        status
                            .cleanup_failures
                            .push("an app-owned destination (cleanup failed)".into());
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::spawn(move || {
                        if let Ok(file) = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(&journal_path)
                        {
                            if file.lock_exclusive().is_ok() {
                                let _ = cleanup_journal_file(file, &journal_path);
                            }
                        }
                    });
                }
                Err(_) => status
                    .cleanup_failures
                    .push("an app-owned destination (lock unavailable)".into()),
            },
            Err(_) => status
                .cleanup_failures
                .push("an app-owned destination (journal unreadable)".into()),
        }
    }
    status
}

fn cleanup_journal_file(file: File, journal_path: &Path) -> io::Result<()> {
    let record: JournalRecord =
        serde_json::from_reader(BufReader::new(&file)).map_err(io::Error::other)?;
    for path in &record.owned_paths {
        validate_owned_name(path, record.run_id)?;
    }
    for path in &record.owned_paths {
        remove_owned(path)?;
    }
    drop(file);
    fs::remove_file(journal_path)
}

fn validate_owned_name(path: &Path, run_id: Uuid) -> io::Result<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "owned path has no UTF-8 basename",
            )
        })?;
    let prefix = format!(".nggedhekake-gambar.{run_id}.tmp.");
    if name.starts_with(&prefix) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "journal path is outside its UUID ownership set",
        ))
    }
}

pub fn validate_output(
    temporary: &Path,
    format: OutputFormat,
    expected_dimensions: [u32; 2],
) -> Result<(), String> {
    let metadata = fs::metadata(temporary).map_err(|error| error.to_string())?;
    if metadata.len() == 0 {
        return Err("Temporary output is empty".into());
    }
    let bytes = fs::read(temporary).map_err(|error| error.to_string())?;
    let actual_format = image::guess_format(&bytes).map_err(|error| error.to_string())?;
    let expected_format = match format {
        OutputFormat::Png => ImageFormat::Png,
        OutputFormat::Jpg => ImageFormat::Jpeg,
        OutputFormat::Webp => ImageFormat::WebP,
    };
    if actual_format != expected_format {
        return Err(format!(
            "Output codec is {actual_format:?}, expected {expected_format:?}"
        ));
    }
    let mut reader = ImageReader::open(temporary)
        .and_then(|reader| reader.with_guessed_format())
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(expected_dimensions[0]);
    limits.max_image_height = Some(expected_dimensions[1]);
    limits.max_alloc = Some(MAX_VALIDATION_ALLOCATION);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    if image.width() != expected_dimensions[0] || image.height() != expected_dimensions[1] {
        return Err(format!(
            "Output dimensions are {}×{}, expected {}×{}",
            image.width(),
            image.height(),
            expected_dimensions[0],
            expected_dimensions[1]
        ));
    }
    Ok(())
}

pub fn publish_exclusive(
    temporary: &Path,
    source: &Path,
    destination: &Path,
    scale: u32,
    format: OutputFormat,
) -> io::Result<PathBuf> {
    File::open(temporary)?.sync_all()?;
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("image");
    for suffix in 1_u64.. {
        let suffix = if suffix == 1 {
            String::new()
        } else {
            format!("-{suffix}")
        };
        let candidate = destination.join(format!(
            "{stem}-upscaled-{scale}x{suffix}.{}",
            format.extension()
        ));
        match rename_exclusive(temporary, &candidate) {
            Ok(()) => {
                File::open(destination)?.sync_all()?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}

#[cfg(target_os = "macos")]
fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
    unsafe extern "C" {
        fn renamex_np(
            from: *const libc::c_char,
            to: *const libc::c_char,
            flags: libc::c_uint,
        ) -> libc::c_int;
    }
    const RENAME_EXCL: libc::c_uint = 0x00000004;
    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "source path contains NUL"))?;
    let destination = CString::new(destination.as_os_str().as_bytes()).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "destination path contains NUL")
    })?;
    let result = unsafe { renamex_np(source.as_ptr(), destination.as_ptr(), RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "macos"))]
fn rename_exclusive(_source: &Path, _destination: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "exclusive publication requires macOS",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    fn write_image(path: &Path, format: ImageFormat, width: u32, height: u32) {
        let image =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, Rgba([1, 2, 3, 255])));
        image.save_with_format(path, format).unwrap();
    }

    #[test]
    fn checked_sizes_enforce_dimension_and_allocation_limits() {
        assert_eq!(checked_output_dimensions((10, 20), 3).unwrap(), [30, 60]);
        assert!(checked_output_dimensions((MAX_OUTPUT_DIMENSION, 1), 2).is_err());
        assert!(checked_output_dimensions((u32::MAX, 1), 2).is_err());
        assert!(rgba_bytes([16_385, 16_385]).unwrap() > MAX_VALIDATION_ALLOCATION);
        assert_eq!(required_space(100), 100 + MIN_SPACE_MARGIN);
    }

    #[test]
    fn disk_warning_uses_the_exact_required_space_threshold() {
        let required = required_space(80 * 1024 * 1024);
        assert!(disk_warning_for_available(required, None).is_none());
        assert!(disk_warning_for_available(required, Some(required)).is_none());
        let warning = disk_warning_for_available(required, Some(required - 1)).unwrap();
        assert_eq!(warning.required_bytes, required);
        assert_eq!(warning.available_bytes, required - 1);
    }

    #[test]
    fn validation_checks_codec_dimensions_and_completeness() {
        let dir = tempdir().unwrap();
        for (extension, format, contract_format) in [
            ("png", ImageFormat::Png, OutputFormat::Png),
            ("jpg", ImageFormat::Jpeg, OutputFormat::Jpg),
            ("webp", ImageFormat::WebP, OutputFormat::Webp),
        ] {
            let path = dir.path().join(format!("valid.{extension}"));
            write_image(&path, format, 4, 6);
            assert!(validate_output(&path, contract_format, [4, 6]).is_ok());
            assert!(validate_output(&path, contract_format, [8, 6]).is_err());
        }
        assert!(
            validate_output(&dir.path().join("valid.png"), OutputFormat::Jpg, [4, 6],).is_err()
        );
        let empty = dir.path().join("empty.png");
        File::create(&empty).unwrap();
        assert!(validate_output(&empty, OutputFormat::Png, [1, 1]).is_err());
        assert!(
            validate_output(&dir.path().join("missing.png"), OutputFormat::Png, [1, 1]).is_err()
        );
        fs::write(dir.path().join("truncated.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        assert!(
            validate_output(&dir.path().join("truncated.png"), OutputFormat::Png, [1, 1]).is_err()
        );
    }

    #[test]
    fn publication_never_overwrites_and_treats_dangling_symlinks_as_occupied() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("photo.png");
        fs::write(&source, b"source").unwrap();
        let base = dir.path().join("photo-upscaled-2x.png");
        let second = dir.path().join("photo-upscaled-2x-2.png");
        fs::write(&base, b"one").unwrap();
        symlink(dir.path().join("missing"), &second).unwrap();
        let temporary = dir.path().join(".nggedhekake-gambar.id.tmp.png");
        fs::write(&temporary, b"new").unwrap();
        let published =
            publish_exclusive(&temporary, &source, dir.path(), 2, OutputFormat::Png).unwrap();
        assert_eq!(published, dir.path().join("photo-upscaled-2x-3.png"));
        assert_eq!(fs::read(base).unwrap(), b"one");
        assert!(second.is_symlink());
        assert_eq!(fs::read(published).unwrap(), b"new");
        assert!(!temporary.exists());
    }

    #[test]
    fn startup_cleanup_removes_only_uuid_owned_paths() {
        let root = tempdir().unwrap();
        let runs = root.path().join("runs");
        let destination = root.path().join("destination");
        fs::create_dir_all(&destination).unwrap();
        let run_id = Uuid::new_v4();
        let owned = temporary_path(&destination, run_id, OutputFormat::Png);
        let unrelated = destination.join("keep.txt");
        fs::write(&owned, b"temporary").unwrap();
        fs::write(&unrelated, b"keep").unwrap();
        let journal =
            OwnedJournal::create(&runs, run_id, UpscaleEngine::Upscayl, vec![owned.clone()])
                .unwrap();
        drop(journal);
        let status = cleanup_startup(&runs);
        assert!(status.cleanup_failures.is_empty());
        assert!(!owned.exists());
        assert_eq!(fs::read(unrelated).unwrap(), b"keep");
    }

    #[test]
    fn locked_journal_waits_then_cleans_after_owner_exits() {
        let root = tempdir().unwrap();
        let runs = root.path().join("runs");
        let destination = root.path().join("destination");
        fs::create_dir_all(&destination).unwrap();
        let run_id = Uuid::new_v4();
        let owned = temporary_path(&destination, run_id, OutputFormat::Png);
        fs::write(&owned, b"temporary").unwrap();
        let journal =
            OwnedJournal::create(&runs, run_id, UpscaleEngine::Upscayl, vec![owned.clone()])
                .unwrap();
        assert!(cleanup_startup(&runs).cleanup_failures.is_empty());
        assert!(owned.exists());
        drop(journal);
        let journal_path = runs.join(format!("{run_id}.json"));
        for _ in 0..100 {
            if !owned.exists() && !journal_path.exists() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(!owned.exists());
        assert!(!journal_path.exists());
    }

    #[test]
    fn corrupt_or_untrusted_journals_are_retained_without_deletion() {
        let root = tempdir().unwrap();
        let runs = root.path().join("runs");
        let destination = root.path().join("destination");
        fs::create_dir_all(&runs).unwrap();
        fs::create_dir_all(&destination).unwrap();
        let corrupt = runs.join("corrupt.json");
        fs::write(&corrupt, b"not json").unwrap();
        let status = cleanup_startup(&runs);
        assert!(!status.cleanup_failures.is_empty());
        assert!(corrupt.exists());

        fs::remove_file(&corrupt).unwrap();
        let run_id = Uuid::new_v4();
        let unrelated = destination.join("final.png");
        fs::write(&unrelated, b"keep").unwrap();
        let journal = OwnedJournal::create(
            &runs,
            run_id,
            UpscaleEngine::Upscayl,
            vec![unrelated.clone()],
        )
        .unwrap();
        drop(journal);
        let status = cleanup_startup(&runs);
        assert!(!status.cleanup_failures.is_empty());
        assert_eq!(fs::read(unrelated).unwrap(), b"keep");
        assert!(runs.join(format!("{run_id}.json")).exists());
    }
}
