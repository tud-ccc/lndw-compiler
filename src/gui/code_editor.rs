use std::collections::HashMap;

use crate::compiler::CompileOptions;
use eframe::egui::{self, Align, Id, Layout, Modifiers};
use rust_i18n::t;

pub enum EditorAction {
    Compile,
    Run,
    Clear,
}

pub struct CodeEditor {
    pub code: String,
    pub compile_options: CompileOptions,
    pub actions: Vec<EditorAction>,
    pub input_variables: HashMap<String, String>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        let mut compile_options = CompileOptions::default();
        compile_options.run_cache_optimization = true;
        Self {
            code: "1 + 1".into(),
            compile_options,
            actions: vec![],
            input_variables: HashMap::new(),
        }
    }
}

impl crate::gui::Window for CodeEditor {
    fn name(&self) -> String {
        "editor.name".into()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(t!(self.name()))
            .id(Id::new(self.name()))
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
            ui.label(t!("editor.explain"));
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

        ui.vertical(|ui| {
            ui.checkbox(
                &mut self.compile_options.do_constant_folding,
                t!("editor.constant_folding"),
            );
            ui.checkbox(
                &mut self.compile_options.run_cache_optimization,
                t!("editor.cache_opt"),
            );
            ui.checkbox(
                &mut self.compile_options.do_common_factor_elimination,
                t!("editor.common_factor_elimination"),
            );
            ui.checkbox(
                &mut self.compile_options.do_shift_replacement,
                t!("editor.replace_mul_with_shift"),
            );
        });

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            if ui.button(t!("editor.compile")).clicked() {
                self.actions.push(EditorAction::Compile);
            }

            if ui
                .button(t!("editor.run"))
                .on_hover_text(t!("editor.run.alt"))
                .clicked()
            {
                self.actions.push(EditorAction::Run);
            }

            if ui.button(t!("editor.clear")).clicked() {
                self.actions.push(EditorAction::Clear);
            }
        });

        if !self.input_variables.is_empty() {
            ui.separator();
            ui.heading(t!("editor.inputs"));

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
