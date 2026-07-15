use dioxus::prelude::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
pub struct AppState {
    pub image_path: Signal<Option<String>>,
    pub output_folder: Signal<Option<String>>,
    pub selected_model: Signal<String>,
    pub scale: Signal<u32>,
    pub format: Signal<String>,
    pub progress: Signal<f32>,
    pub progress_message: Signal<String>,
    pub upscaled_image_path: Signal<Option<String>>,
    pub is_processing: Signal<bool>,
    pub models: Signal<Vec<String>>,
    pub error: Signal<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            image_path: use_signal(|| None),
            output_folder: use_signal(|| None),
            selected_model: use_signal(|| "realesrgan-x4plus".into()),
            scale: use_signal(|| 4),
            format: use_signal(|| "png".into()),
            progress: use_signal(|| 0.0),
            progress_message: use_signal(String::new),
            upscaled_image_path: use_signal(|| None),
            is_processing: use_signal(|| false),
            models: use_signal(Vec::new),
            error: use_signal(|| None),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = convertFileSrc)]
    fn convert_file_src(path: &str) -> String;
}

pub async fn invoke_command<T: Serialize>(command: &str, args: &T) -> Result<JsValue, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    tauri_invoke(command, args)
        .await
        .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))
}

pub fn image_url(path: &str) -> String {
    convert_file_src(path)
}
