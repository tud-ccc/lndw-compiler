use std::collections::HashMap;
use eframe::egui;
use crate::compiler::{compile, interpret_ir, Inst, Reg};
use crate::gui::CodeEditor;

#[derive(Default)]
pub struct LndwApp {
    code_editor: CodeEditor,
    asm_output: Vec<(Option<f32>, Inst)>,
    variable_mapping: HashMap<String, (Reg, String)>,
    result: Option<i32>,
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
            .default_width(300.0)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Output");
                });

                ui.horizontal_centered(|ui| {
                    egui::Grid::new("output").min_col_width(50.0).show(ui, |ui| {
                        let mut should_update = true;
                        for (progress, inst) in &mut self.asm_output {
                            if should_update && progress.is_none() {
                                *progress = Some(0.03);
                                should_update = false;
                            }
                            if should_update && progress.is_some() && progress.unwrap() < 1.0 {
                                *progress = Some(progress.unwrap() + 0.03);
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
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            
            self.code_editor.ui(ui);

            if ui.button("Compile!").clicked() {
                let (asm, vars) = compile(&self.code_editor.code)
                    .unwrap_or_else(|e| (vec![], HashMap::new()));
                self.asm_output = asm.iter().map(|i| (None, i.clone())).collect();
                self.variable_mapping = vars.iter().map(|(k, &v)| (k.clone(), (v, String::new()))).collect();
            }

            egui::Grid::new("vars").show(ui, |ui| {
                for (var, val) in self.variable_mapping.iter_mut() {
                    ui.label(var);
                    ui.text_edit_singleline(&mut val.1);
                    ui.end_row();
                }
            });
            
            if ui.button("Run!").clicked() {
                let result = interpret_ir(self.asm_output.iter().map(|i| i.1.clone()).collect(), &self.variable_mapping);
                self.result = result.ok();
            }
            
            if let Some(result) = self.result {
                ui.label(format!("Result: {result}"));
            }
        });
    }
}
