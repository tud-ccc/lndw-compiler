use eframe::egui;
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
        Self::default()
    }
}

impl eframe::App for LndwApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Right Panel");
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    "asdfasdf\nasdfasdfasdfasdf\nasdfasdf".to_string()
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            
            self.code_editor.ui(ui);

            self.asm_output = "asdfasdf\nasdfasdf".to_string();

            egui::ScrollArea::vertical().id_salt("out").show(ui, |ui| {
                if ui.button("Click me!").clicked() {
                    println!("Click me!");
                }
                ui.add(
                    egui::TextEdit::multiline(&mut self.asm_output.as_str())
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .desired_width(f32::INFINITY)
                );
            });
        });
    }
}
