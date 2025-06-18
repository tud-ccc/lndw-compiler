mod app;
mod compiler;
mod gui;
mod parser;
mod passes;
mod types;

use crate::app::LndwApp;

const APP_NAME: &str = "Lange Nacht der Wissenschaften";

// TODO(feliix42): could be a setting in the UI?
const REGISTER_COUNT: u8 = 6;
const RAM_SIZE: usize = 16;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(LndwApp::new(cc)))),
    );
}
