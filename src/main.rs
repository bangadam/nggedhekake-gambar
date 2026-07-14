use std::{cell::RefCell, path::PathBuf, rc::Rc};

use nggedhekake_gambar::{
    AppWindow,
    controller::{WorkspaceController, WorkspaceState},
    source::ImageSourceLoader,
};
use slint::{
    ComponentHandle, Rgba8Pixel, SharedPixelBuffer,
    winit_030::{EventResult, WinitWindowAccessor, winit::event::WindowEvent},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    let ui = AppWindow::new()?;
    let controller = Rc::new(RefCell::new(WorkspaceController::new(ImageSourceLoader)));

    let picker_ui = ui.as_weak();
    let picker_controller = Rc::clone(&controller);
    ui.on_choose_image(move || {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };

        if let Some(ui) = picker_ui.upgrade() {
            select_source(&ui, &picker_controller, path);
        }
    });

    let drop_ui = ui.as_weak();
    let drop_controller = Rc::clone(&controller);
    ui.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::DroppedFile(path) = event {
            if let Some(ui) = drop_ui.upgrade() {
                select_source(&ui, &drop_controller, path.clone());
            }
        }

        EventResult::Propagate
    });

    ui.run()?;
    Ok(())
}

fn select_source(
    ui: &AppWindow,
    controller: &Rc<RefCell<WorkspaceController<ImageSourceLoader>>>,
    path: PathBuf,
) {
    controller.borrow_mut().select_source(&path);
    render_state(ui, controller.borrow().state());
}

fn render_state(ui: &AppWindow, state: &WorkspaceState) {
    match state {
        WorkspaceState::Empty => {
            ui.set_has_source(false);
            ui.set_has_error(false);
            ui.set_status_text("Choose an image to begin".into());
        }
        WorkspaceState::Ready(source) => {
            let mut pixels =
                SharedPixelBuffer::<Rgba8Pixel>::new(source.preview.width, source.preview.height);
            pixels
                .make_mut_bytes()
                .copy_from_slice(&source.preview.rgba);

            ui.set_preview_image(slint::Image::from_rgba8(pixels));
            ui.set_source_name(source.name.clone().into());
            ui.set_source_path(source.path.display().to_string().into());
            ui.set_source_details(source.details().into());
            ui.set_error_message("".into());
            ui.set_has_error(false);
            ui.set_has_source(true);
            ui.set_status_text("Image ready".into());
        }
        WorkspaceState::Error(message) => {
            ui.set_preview_image(slint::Image::default());
            ui.set_source_name("".into());
            ui.set_source_path("".into());
            ui.set_source_details("".into());
            ui.set_error_message(message.clone().into());
            ui.set_has_source(false);
            ui.set_has_error(true);
            ui.set_status_text("Image could not be opened".into());
        }
    }
}
