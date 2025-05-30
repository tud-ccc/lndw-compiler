use eframe::egui;
use crate::compiler::compile;
use crate::gui::CodeEditor;

#[derive(Default)]
pub struct LndwApp {
    code_editor: CodeEditor,
    asm_output: String,
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
            .default_width(200.0)
            .width_range(80.0..=300.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Output");
                });

                ui.horizontal_centered(|ui| {
                    egui::ScrollArea::vertical().id_salt("out").show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.asm_output.as_str())
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .desired_width(f32::INFINITY)
                        );
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            
            self.code_editor.ui(ui);

            if ui.button("Click me!").clicked() {
                self.asm_output = compile(&self.code_editor.code).unwrap_or_else(|e| format!("Error: {e}"))
            }
        });
    }
}
