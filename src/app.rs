use std::collections::{HashMap, HashSet};
use eframe::egui;
use crate::compiler::{compile, interpret_ir, Inst};
use crate::gui::CodeEditor;

#[derive(Default)]
pub struct LndwApp {
    code_editor: CodeEditor,
    asm_output: Vec<(Option<f32>, Inst)>,
    input_variables: HashMap<String, String>,
    result: Option<String>,
}

impl LndwApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        cc.egui_ctx.set_zoom_factor(1.5);
        Self::default()
    }
}

impl eframe::App for LndwApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Output");
                });

                egui::Grid::new("output").min_col_width(50.0).show(ui, |ui| {
                    let mut should_update = true;
                    for (progress, inst) in &mut self.asm_output {
                        let progress_increment = match inst {
                            Inst::Add(_, _) => 0.05,
                            Inst::Sub(_, _) => 0.05,
                            Inst::Mul(_, _) => 0.025,
                            Inst::Div(_, _) => 0.01,
                            Inst::Store(_, _) => 0.1,
                            Inst::Transfer(_, _) => 0.1,
                            Inst::Result(_) => 0.1,
                        };
                        if should_update && progress.is_none() {
                            *progress = Some(progress_increment);
                            should_update = false;
                        }
                        if should_update && progress.is_some() && progress.unwrap() < 1.0 {
                            *progress = Some(progress.unwrap() + progress_increment);
                            should_update = false;
                        }
                        let bar = egui::ProgressBar::new(progress.unwrap_or(0.0))
                            .animate(true)
                            .desired_width(50.0)
                            .desired_height(7.5);
                        ui.add_visible(progress.is_some(), bar);
                        ui.label(format!("{inst}"));
                        ui.end_row();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello! Write some expression here");
            
            self.code_editor.ui(ui);

            if ui.button("Compile!").clicked() {
                self.result = None;
                let (asm, vars) = compile(&self.code_editor.code)
                    .unwrap_or_else(|e| {
                        self.result = Some(format!("error: {e}"));
                        (vec![], HashSet::new())
                    });
                self.asm_output = asm.iter().map(|i| (None, i.clone())).collect();
                self.input_variables = vars.iter().map(|s| (s.clone(), String::new())).collect();
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
                let result = interpret_ir(self.asm_output.iter().map(|i| i.1.clone()).collect(), &self.input_variables);
                self.result = match result {
                    Ok(r) => Some(r.to_string()),
                    Err(e) => Some(format!("error: {e}")),
                }
            }
            
            if let Some(result) = &self.result {
                ui.label(format!("Result: {result}"));
            }
        });
    }
}
