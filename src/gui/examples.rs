use eframe::egui::{self, Label, Layout, RichText, Widget};
use eframe::egui::{Align, Id};
use rust_i18n::t;

use crate::compiler::CompileOptions;

pub struct Example {
    title: &'static str,
    desc: &'static str,
    pub input: &'static str,
    pub options: CompileOptions,
}

#[derive(Default)]
pub struct Examples {
    pub examples: Vec<Example>,
    pub chosen: Option<usize>,
}

impl crate::gui::Window for Examples {
    fn name(&self) -> String {
        "examples.name".into()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(t!(self.name()))
            .id(Id::new(self.name()))
            .default_width(320.0)
            .default_height(400.0)
            .open(open)
            .resizable([false, false])
            .scroll(true)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl Examples {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        for (i, example) in self.examples.iter().enumerate() {
            if i > 0 {
                ui.separator();
            }

            egui::CollapsingHeader::new(t!(example.title))
                .id_salt(example.title)
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(t!(example.desc));
                    ui.add_space(9.0);

                    ui.vertical_centered(|ui| {
                        Label::new(RichText::new(example.input).monospace())
                            .selectable(false)
                            .ui(ui);
                    });
                    ui.add_space(5.0);

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button(t!("examples.use")).clicked() {
                            self.chosen = Some(i);
                        }
                    });
                });
        }
    }

    pub fn preloaded() -> Self {
        let mut res = Self::default();

        res.examples.push(Example {
            title: "examples.basic.title",
            desc: "examples.basic.desc",
            input: "3 + 2 + 1",
            options: CompileOptions {
                do_constant_folding: false,
                run_cache_optimization: false,
                do_common_factor_elimination: false,
                do_shift_replacement: false,
            },
        });

        res.examples.push(Example {
            title: "examples.complex.title",
            desc: "examples.complex.desc",
            input: "1000 * 2 + 4 * 5 + (15 / 3) + x * 13 - y * 2",
            options: CompileOptions {
                do_constant_folding: true,
                run_cache_optimization: false,
                do_common_factor_elimination: false,
                do_shift_replacement: false,
            },
        });

        res.examples.push(Example {
            title: "examples.ram_opt.title",
            desc: "examples.ram_opt.desc",
            input: "(1000 + 2) * (4 * 5 + (15 / 3) + 17 * 13 - 8 * 2)",
            options: CompileOptions {
                do_constant_folding: false,
                run_cache_optimization: true,
                do_common_factor_elimination: false,
                do_shift_replacement: false,
            },
        });

        res.examples.push(Example {
            title: "examples.shift_mul.title",
            desc: "examples.shift_mul.desc",
            input: "16 / 2 * 4 / 4",
            options: CompileOptions {
                do_constant_folding: false,
                run_cache_optimization: true,
                do_common_factor_elimination: false,
                do_shift_replacement: true,
            },
        });

        res.examples.push(Example {
            title: "examples.factorization.title",
            desc: "examples.factorization.desc",
            input: "t * 16 + t * (3 + 2)",
            options: CompileOptions {
                do_constant_folding: false,
                run_cache_optimization: true,
                do_common_factor_elimination: true,
                do_shift_replacement: false,
            },
        });

        res
    }
}
