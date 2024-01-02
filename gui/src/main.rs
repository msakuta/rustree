mod app;
mod data;

use crate::{app::RustreeApp};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut native_options = eframe::NativeOptions::default();

    // We insist to use light theme, because the canvas color is designed to work with light background.
    native_options.follow_system_theme = false;
    native_options.default_theme = eframe::Theme::Light;

    eframe::run_native(
        "rustree GUI",
        native_options,
        Box::new(|_cc| Box::new(RustreeApp::new())),
    )
    .unwrap();
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
    eframe::WebRunner::new()
        .start(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|_cc| Box::new(RustreeApp::new())),
        )
        .await
        .expect("failed to start eframe");
    });
}
