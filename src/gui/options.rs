use eframe::egui;

#[derive(Copy, Clone)]
pub struct InterpreterOptions {
    pub num_registers: u8,
    pub num_cachelines: usize,
}

impl Default for InterpreterOptions {
    fn default() -> Self {
        Self {
            num_registers: 6,
            num_cachelines: 16,
        }
    }
}

impl crate::gui::Window for InterpreterOptions {
    fn name(&self) -> String {
        "Interpreter Options".into()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .default_width(320.0)
            .default_height(400.0)
            .open(open)
            .resizable([false, false])
            .scroll(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl InterpreterOptions {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("These are the settings for the Interpreter executing the code. Change them and see how the compile results change with more or less cache lines or registers.");

        ui.add_space(12.0);

        egui::Grid::new("reg_count")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Register Count");
                let mut reg_count = format!("{}", self.num_registers);
                ui.text_edit_singleline(&mut reg_count);
                if let Ok(val) = reg_count.parse() {
                    self.num_registers = val;
                }
                ui.end_row();
            });

        let reg_expl = "A computer processor can remember a fixed number of things. \
            They work like post-its on your monitor. \
            There's space for a few and you have them always ready.";
        egui::CollapsingHeader::new("Explanation")
            .id_salt(reg_expl)
            .default_open(true)
            .show(ui, |ui| {
                ui.label(reg_expl);
            });

        ui.add_space(12.0);

        egui::Grid::new("num_cachelines")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Cache Size");
                let mut num_cachelines = format!("{}", self.num_cachelines);
                ui.text_edit_singleline(&mut num_cachelines);
                if let Ok(val) = num_cachelines.parse() {
                    self.num_cachelines = val;
                }
                ui.end_row();
            });

        let cache_expl = "If all registers are full, computers have larger storages, called caches. \
            While they're bigger, they're also slower to access. Think of them like big \
            binders of files. They can hold a lot of paper, but finding a specific page takes time.";

        egui::CollapsingHeader::new("Explanation")
            .id_salt(cache_expl)
            .default_open(true)
            .show(ui, |ui| {
                ui.label(cache_expl);
            });
    }
}
