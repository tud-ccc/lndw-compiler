use eframe::egui;

pub struct CodeEditor {
    pub code: String,
    do_constant_folding: bool,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "1 + 1".into(),
            do_constant_folding: false,
        }
    }
}

impl CodeEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(0.0);
            ui.label("You can write your expressions in this TextEdit box.");
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.do_constant_folding, "Constant folding");
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
            );
        });
    }
}
