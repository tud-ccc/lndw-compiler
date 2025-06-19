mod assembly_output;
mod code_editor;

pub use assembly_output::*;
pub use code_editor::*;
use eframe::egui;

pub trait Window {
    /// Name of the window
    fn name(&self) -> String;

    /// Show the window, depending on `open`.
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}
