use eframe::egui;
use eframe::egui::Id;
use rust_i18n::t;

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
        "interp_opts.name".into()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(t!(self.name()))
            .id(Id::new(self.name()))
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
        ui.label(t!("interp_opts.label"));

        ui.add_space(12.0);

        egui::Grid::new("reg_count")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(t!("interp_opts.n_regs"));
                let mut reg_count = format!("{}", self.num_registers);
                ui.text_edit_singleline(&mut reg_count);
                if let Ok(val) = reg_count.parse() {
                    self.num_registers = val;
                }
                ui.end_row();
            });

        egui::CollapsingHeader::new(t!("interp_opts.explanation"))
            .id_salt("interp_opts.label_regs")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(t!("interp_opts.label_regs"));
            });

        ui.add_space(12.0);

        egui::Grid::new("num_cachelines")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(t!("interp_opts.cache_size"));
                let mut num_cachelines = format!("{}", self.num_cachelines);
                ui.text_edit_singleline(&mut num_cachelines);
                if let Ok(val) = num_cachelines.parse() {
                    self.num_cachelines = val;
                }
                ui.end_row();
            });

        egui::CollapsingHeader::new(t!("interp_opts.explanation"))
            .id_salt("interp_opts.cache_label")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(t!("interp_opts.cache_label"));
            });
    }
}
