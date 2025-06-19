mod app;
mod compiler;
mod gui;
mod parser;
mod passes;
mod types;

use crate::app::LndwApp;
use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        &t!("app.name"),
        native_options,
        Box::new(|cc| Ok(Box::new(LndwApp::new(cc)))),
    );
}
