use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, State, Theme, WebviewWindow};

#[derive(Default)]
pub struct StartupState(Arc<AtomicBool>);

fn prepare_window(window: &WebviewWindow, light: bool) {
    let theme = if light { Theme::Light } else { Theme::Dark };
    let color = if light {
        (255, 249, 239, 255)
    } else {
        (20, 19, 16, 255)
    };
    let _ = window.set_background_color(Some(tauri::window::Color(
        color.0, color.1, color.2, color.3,
    )));
    let _ = window.set_theme(Some(theme));
}

#[tauri::command]
pub fn show_main_window(
    window: WebviewWindow,
    state: State<'_, StartupState>,
    theme: String,
) -> Result<(), String> {
    prepare_window(&window, theme == "light");
    window.show().map_err(|error| error.to_string())?;
    state.0.store(true, Ordering::Release);
    Ok(())
}

pub fn setup(app: &tauri::App) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    prepare_window(&window, window.theme().ok() == Some(Theme::Light));
    let ready = app.state::<StartupState>().0.clone();
    // A failed frontend must still leave a visible, recoverable startup page.
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(8));
        if !ready.load(Ordering::Acquire) {
            let _ = window.show();
        }
    });
}
