use crate::gui::{AssemblyOutput, CodeEditor};
use eframe::egui::{self, FontData, FontFamily};
use eframe::epaint::text::{FontInsert, InsertFontFamily};
use std::collections::HashMap;

#[derive(Default)]
pub struct LndwApp {
    code_editor: CodeEditor,
    asm_unoptimized: AssemblyOutput,
    asm_optimized: AssemblyOutput,
    input_variables: HashMap<String, String>,
    result: Option<String>,
}

impl LndwApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        cc.egui_ctx.set_zoom_factor(1.5);

        cc.egui_ctx.add_font(FontInsert::new(
            "Iosevka Term Regular",
            FontData::from_static(include_bytes!("../assets/IosevkaTerm-Regular.ttf")),
            vec![InsertFontFamily {
                family: FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));

        Self {
            asm_unoptimized: AssemblyOutput::empty("Unoptimized output".to_string()),
            asm_optimized: AssemblyOutput::empty("Optimized output".to_string()),
            ..Self::default()
        }
    }
}

impl eframe::App for LndwApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("panel_unoptimized")
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                self.asm_optimized.ui(ui);
            });

        egui::SidePanel::right("panel_optimized")
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                self.asm_unoptimized.ui(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello! Write some expression here");

            self.code_editor.ui(ui);

            if ui.button("Compile!").clicked() {
                if let Ok(vars) = self.asm_unoptimized.compile(&self.code_editor.code, false) {
                    self.input_variables =
                        vars.iter().map(|s| (s.clone(), String::new())).collect();
                } else {
                    self.input_variables.clear();
                }

                if self.code_editor.do_constant_folding {
                    // TODO: consider what to do with vars & any errors.
                    let _ = self.asm_optimized.compile(&self.code_editor.code, true);
                }
            }

            if !self.input_variables.is_empty() {
                ui.heading("Input variables:");
            }

            egui::Grid::new("vars").show(ui, |ui| {
                for (var, val) in self.input_variables.iter_mut() {
                    ui.label(var);
                    ui.text_edit_singleline(val);
                    ui.end_row();
                }
            });

            if ui.button("Run!").clicked() {
                self.asm_unoptimized.run(&self.input_variables);
                self.asm_optimized.run(&self.input_variables);
            }

            if let Some(result) = &self.result {
                ui.label(format!("Result: {result}"));
            }

            if ui.button("Clear").clicked() {
                self.asm_unoptimized.clear();
                self.asm_optimized.clear();
                self.result = None;
            }
        });
    }
}
