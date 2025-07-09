mod app;
mod compiler;
mod gui;
mod interpreter;
mod parser;
mod passes;
mod types;

use std::sync::Arc;
use crate::app::LndwApp;
use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");

const APP_NAME: &str = "tud-ccc-demo-compiler";

fn main() {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"));
    let icon = icon.ok().map(|i| Arc::new(i));
    if icon.is_none() {
        eprintln!("Warning: failed to load icon");
    }

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.maximized = Some(true);
    native_options.viewport.title = Some(t!("app.default_name").into());
    native_options.viewport.icon = icon;

    let _ = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(LndwApp::new(cc)))),
    );
}
