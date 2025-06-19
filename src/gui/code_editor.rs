use std::collections::HashMap;

use eframe::egui::{self, Align, Layout, Modifiers};

pub enum EditorAction {
    Compile,
    Run,
    Clear,
}

pub struct CodeEditor {
    pub code: String,
    pub do_constant_folding: bool,
    pub optimize_cache_use: bool,
    pub actions: Vec<EditorAction>,
    pub input_variables: HashMap<String, String>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "1 + 1".into(),
            do_constant_folding: false,
            optimize_cache_use: true,
            actions: vec![],
            input_variables: HashMap::new(),
        }
    }
}

impl crate::gui::Window for CodeEditor {
    fn name(&self) -> String {
        "🖮 Code Editor".into()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl CodeEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let compile_run_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::Enter);

        if ui.input_mut(|i| i.consume_shortcut(&compile_run_shortcut)) {
            self.actions.push(EditorAction::Compile);
            self.actions.push(EditorAction::Run);
        }

        ui.horizontal(|ui| {
            ui.set_height(0.0);
            ui.label("You can write your expressions in this TextEdit box.");
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY),
            );
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.do_constant_folding, "Constant folding");
            ui.checkbox(&mut self.optimize_cache_use, "Cache Optimization");
        });

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            if ui.button("Compile!").clicked() {
                self.actions.push(EditorAction::Compile);
            }

            if ui
                .button("Run!")
                .on_hover_text("You can alternatively press CTRL+Enter to compile and run")
                .clicked()
            {
                // TODO: Cmd enter
                self.actions.push(EditorAction::Run);
            }

            if ui.button("Clear").clicked() {
                self.actions.push(EditorAction::Clear);
            }
        });

        if !self.input_variables.is_empty() {
            ui.separator();
            ui.heading("Input variables:");

            // TODO: look nicer
            egui::Grid::new("vars")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for (var, val) in self.input_variables.iter_mut() {
                        ui.label(var);
                        ui.text_edit_singleline(val);
                        ui.end_row();
                    }
                });
        }
    }
}
