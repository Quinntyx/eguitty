pub mod model;
pub mod gui;
pub mod app;

use app::App;


fn main () -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "eguitty 0.1.0",
        options,
        Box::new(App::setup),
    )
}

