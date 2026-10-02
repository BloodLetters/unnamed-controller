#![warn(clippy::all, rust_2018_idioms)]

/// Initializes the web assembly logger and starts the eframe web application.
#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Box::new(sim_ui::SimulatorApp::new(cc))),
            )
            .await;

        let loading_text = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("loading_text"));
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => loading_text.remove(),
                Err(error) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {error:?}");
                }
            }
        }
    });
}

/// Fallback for when mistakenly compiled natively.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Use `app-desktop` to run native, not `app-web`.");
}
