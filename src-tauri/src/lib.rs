use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

struct ActiveProcess {
    id: u32,
    child: Arc<Mutex<Child>>,
}

#[derive(Default)]
struct ProcessState(Mutex<Option<ActiveProcess>>);

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum UpscaleEngine {
    Upscayl,
    RealEsrgan,
}

impl UpscaleEngine {
    fn binary_resource(&self) -> &'static str {
        match self {
            Self::Upscayl => "binaries/upscayl-bin",
            Self::RealEsrgan => "binaries/realesrgan-ncnn-vulkan",
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    percent: f32,
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DonePayload {
    output_path: String,
}

#[derive(Clone, Serialize)]
struct ErrorPayload {
    message: String,
}

fn resource_path(app: &AppHandle, relative: &str) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resolve(relative, BaseDirectory::Resource)
        .map_err(|error| error.to_string())?;

    if bundled.exists() {
        return Ok(bundled);
    }

    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if development.exists() {
        Ok(development)
    } else {
        Err(format!("Bundled resource not found: {relative}"))
    }
}

fn selected_path(path: tauri_plugin_dialog::FilePath) -> Option<String> {
    path.into_path()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
async fn select_image(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
        .blocking_pick_file()
        .and_then(selected_path)
}

#[tauri::command]
async fn select_folder(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .and_then(selected_path)
}

#[tauri::command]
fn get_models(app: AppHandle) -> Result<Vec<String>, String> {
    let models_path = resource_path(&app, "models")?;
    let mut models = fs::read_dir(models_path)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()?.to_str()? == "param")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))
                .flatten()
        })
        .filter(|model| {
            resource_path(&app, &format!("models/{model}.bin")).is_ok_and(|path| path.is_file())
        })
        .collect::<Vec<_>>();
    models.sort();
    Ok(models)
}

fn parse_progress(line: &str) -> Option<f32> {
    line.split_whitespace().find_map(|word| {
        if !word.contains('%') {
            return None;
        }
        word.trim_matches(|character: char| !character.is_ascii_digit() && character != '.')
            .parse::<f32>()
            .ok()
            .map(|value| value.clamp(0.0, 100.0))
    })
}

fn output_file(input: &Path, folder: &Path, scale: u32, format: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    folder.join(format!("{stem}-upscaled-{scale}x.{format}"))
}

#[tauri::command]
fn upscale_image(
    app: AppHandle,
    state: State<'_, ProcessState>,
    image_path: String,
    output_folder: String,
    engine: UpscaleEngine,
    model: String,
    scale: u32,
    format: String,
    gpu_id: Option<i32>,
) -> Result<String, String> {
    if !matches!(scale, 2 | 3 | 4) {
        return Err("Scale must be 2, 3, or 4".into());
    }
    if !matches!(format.as_str(), "png" | "jpg" | "webp") {
        return Err("Format must be png, jpg, or webp".into());
    }

    let input = PathBuf::from(&image_path);
    if !input.is_file() {
        return Err("The selected image no longer exists".into());
    }

    let folder = PathBuf::from(&output_folder);
    if !folder.is_dir() {
        return Err("The selected output folder no longer exists".into());
    }

    let models_path = resource_path(&app, "models")?;
    if !models_path.join(format!("{model}.param")).is_file()
        || !models_path.join(format!("{model}.bin")).is_file()
    {
        return Err(format!("Model is not available: {model}"));
    }

    let binary_path = resource_path(&app, engine.binary_resource())?;
    let output = output_file(&input, &folder, scale, &format);

    let mut process = Command::new(binary_path);
    process.args([
        "-i",
        input
            .to_str()
            .ok_or_else(|| "Image path is not valid UTF-8".to_string())?,
        "-o",
        output
            .to_str()
            .ok_or_else(|| "Output path is not valid UTF-8".to_string())?,
        "-s",
        &scale.to_string(),
        "-m",
        models_path
            .to_str()
            .ok_or_else(|| "Models path is not valid UTF-8".to_string())?,
        "-n",
        &model,
        "-f",
        &format,
    ]);
    if let Some(gpu_id) = gpu_id {
        process.args(["-g", &gpu_id.to_string()]);
    }

    let mut child = process
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start upscaler: {error}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to read upscaler progress".to_string())?;
    let process_id = child.id();
    let child = Arc::new(Mutex::new(child));

    {
        let mut active = state
            .0
            .lock()
            .map_err(|_| "Upscaler process state is unavailable".to_string())?;
        if active.is_some() {
            let _ = child.lock().map(|mut process| process.kill());
            return Err("An upscale is already running".into());
        }
        *active = Some(ActiveProcess {
            id: process_id,
            child: Arc::clone(&child),
        });
    }

    let output_string = output.to_string_lossy().into_owned();
    let event_output = output_string.clone();
    let app_handle = app.clone();

    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Some(percent) = parse_progress(&line) {
                let _ = app_handle.emit(
                    "upscale-progress",
                    ProgressPayload {
                        percent,
                        message: line,
                    },
                );
            }
        }

        let status = child
            .lock()
            .ok()
            .and_then(|mut process| process.wait().ok());
        if status.is_some_and(|status| status.success()) && Path::new(&event_output).is_file() {
            let _ = app_handle.emit(
                "upscale-done",
                DonePayload {
                    output_path: event_output,
                },
            );
        } else {
            let message = status
                .map(|status| format!("Upscaler exited with status {status}"))
                .unwrap_or_else(|| "Upscaler process could not be completed".into());
            let _ = app_handle.emit("upscale-error", ErrorPayload { message });
        }

        if let Ok(mut active) = app_handle.state::<ProcessState>().0.lock() {
            if active
                .as_ref()
                .is_some_and(|process| process.id == process_id)
            {
                *active = None;
            }
        }
    });

    Ok(output_string)
}

#[tauri::command]
fn stop_upscale(state: State<'_, ProcessState>) -> Result<(), String> {
    let child = {
        let active = state
            .0
            .lock()
            .map_err(|_| "Upscaler process state is unavailable".to_string())?;
        active
            .as_ref()
            .map(|process| Arc::clone(&process.child))
            .ok_or_else(|| "No upscale is running".to_string())?
    };

    let result = child
        .lock()
        .map_err(|_| "Upscaler process is unavailable".to_string())?
        .kill()
        .map_err(|error| format!("Failed to stop upscaler: {error}"));
    result
}

#[tauri::command]
fn open_folder(app: AppHandle, path: String) -> Result<(), String> {
    if !Path::new(&path).is_dir() {
        return Err("Output folder does not exist".into());
    }
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(ProcessState::default())
        .invoke_handler(tauri::generate_handler![
            select_image,
            select_folder,
            upscale_image,
            stop_upscale,
            get_models,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::UpscaleEngine;

    #[test]
    fn engine_names_resolve_to_their_bundled_executables() {
        let upscayl: UpscaleEngine = serde_json::from_str("\"upscayl\"").unwrap();
        let official: UpscaleEngine = serde_json::from_str("\"real-esrgan\"").unwrap();

        assert_eq!(upscayl.binary_resource(), "binaries/upscayl-bin");
        assert_eq!(
            official.binary_resource(),
            "binaries/realesrgan-ncnn-vulkan"
        );
    }

    #[test]
    fn unknown_engine_name_is_rejected() {
        assert!(serde_json::from_str::<UpscaleEngine>("\"unknown\"").is_err());
    }
}
