mod upscale;

use std::path::{Path, PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use upscale::UpscaleService;
use upscale_contract::{
    RetryUpscaleRequest, RunEvent, StartOutcome, StartUpscaleRequest, StartupStatus, UpscaleEngine,
};

fn resources_root(app: &AppHandle) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resolve(".", BaseDirectory::Resource)
        .map_err(|error| error.to_string())?;
    if bundled.join("models").is_dir() {
        return Ok(bundled);
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if development.join("models").is_dir() {
        Ok(development)
    } else {
        Err("Bundled upscaling resources are unavailable".into())
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
fn get_models(service: State<'_, UpscaleService>) -> Result<Vec<String>, String> {
    service.models()
}

#[tauri::command]
fn start_upscale(
    service: State<'_, UpscaleService>,
    request: StartUpscaleRequest,
) -> Result<StartOutcome, String> {
    service.start(request)
}

#[tauri::command]
fn retry_upscale(
    service: State<'_, UpscaleService>,
    request: RetryUpscaleRequest,
) -> Result<StartOutcome, String> {
    service.retry(request)
}

#[tauri::command]
fn cancel_upscale(service: State<'_, UpscaleService>, run_id: String) -> Result<(), String> {
    service.cancel(&run_id)
}

#[tauri::command]
fn get_upscale_status(service: State<'_, UpscaleService>) -> Option<RunEvent> {
    service.status()
}

#[tauri::command]
fn get_engine_preference(service: State<'_, UpscaleService>) -> Result<UpscaleEngine, String> {
    service.engine_preference()
}

#[tauri::command]
fn set_engine_preference(
    service: State<'_, UpscaleService>,
    engine: UpscaleEngine,
) -> Result<UpscaleEngine, String> {
    service.set_engine_preference(engine)
}

#[tauri::command]
fn get_startup_status(service: State<'_, UpscaleService>) -> StartupStatus {
    service.startup_status()
}

#[tauri::command]
fn copy_technical_details(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| error.to_string())
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let resources = resources_root(&handle)?;
            let app_data_dir = handle
                .path()
                .app_local_data_dir()
                .map_err(|error| error.to_string())?;
            let config_dir = handle
                .path()
                .app_config_dir()
                .map_err(|error| error.to_string())?;
            app.manage(UpscaleService::new(
                handle,
                resources,
                app_data_dir,
                config_dir,
            ));
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                window.state::<UpscaleService>().cancel_active();
            }
        })
        .invoke_handler(tauri::generate_handler![
            select_image,
            select_folder,
            get_models,
            start_upscale,
            retry_upscale,
            cancel_upscale,
            get_upscale_status,
            get_engine_preference,
            set_engine_preference,
            get_startup_status,
            copy_technical_details,
            open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
