mod assembly_output;
mod code_editor;
mod examples;
mod options;

pub use assembly_output::*;
pub use code_editor::*;
use eframe::egui;
pub use examples::*;
pub use options::*;

pub trait Window {
    /// Name of the window
    fn name(&self) -> String;

    /// Show the window, depending on `open`.
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}
