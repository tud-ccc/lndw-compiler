use crate::{
    compiler::{CompileOptions, Compiler, Inst, u8tochar},
    gui::InterpreterOptions,
    interpreter::Interpreter,
};
use eframe::egui::Id;
use eframe::egui::{self, Widget};
use rust_i18n::t;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct AssemblyOutput {
    heading: String,
    asm: Option<Vec<(Inst, f32)>>,
    error: Option<String>,
    program_result: Option<i32>,
    interpreter: Option<Interpreter>,
    hw: Option<InterpreterOptions>,
    running: bool,
    total_time: f32,
}

impl AssemblyOutput {
    /// Construct empty UI with a name.
    pub fn empty(heading: String) -> Self {
        Self {
            heading,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn new(heading: String, asm: Vec<Inst>) -> Self {
        Self {
            heading,
            asm: Some(asm.iter().map(|i| (i.clone(), 0.0)).collect()),
            ..Default::default()
        }
    }

    /// Clear any assembly and error message.
    pub fn clear(&mut self) {
        self.asm = None;
        self.error = None;
        self.program_result = None;
        self.running = false;
        self.total_time = 0.0;
        self.hw = None;
        self.interpreter = None;
    }

    pub fn instructions(&self) -> Vec<Inst> {
        self.asm
            .as_ref()
            .map_or(vec![], |v| v.iter().map(|(inst, _)| inst.clone()).collect())
    }

    pub fn compile(
        &mut self,
        input: &str,
        opts: CompileOptions,
        hw: InterpreterOptions,
    ) -> Result<HashSet<String>, ()> {
        self.clear();
        let r = Compiler::with(opts).with_interpreter(hw).compile(input);
        self.hw = Some(hw);

        r.map(|(asm, vars)| {
            self.asm = Some(asm.iter().map(|i| (i.clone(), 0.0)).collect());
            vars
        })
        .map_err(|e| {
            self.error = Some(format!("Compile error: {e}"));
        })
    }

    pub fn run(&mut self, vars: &HashMap<String, String>) {
        self.program_result = None;

        if self.asm.is_none() {
            return;
        }

        let hw = self.hw.unwrap();

        match Interpreter::with_config(&hw)
            .load_instructions(self.instructions())
            .with_variables(vars.to_owned())
            .run_to_end()
        {
            Ok(r) => {
                self.program_result = Some(r);
                self.running = true;
                // don't overwrite the interpreter
                if self.interpreter.is_none() {
                    self.interpreter = Some(
                        Interpreter::with_config(&hw)
                            .load_instructions(self.instructions())
                            .with_variables(vars.to_owned()),
                    );
                }
            }
            Err(e) => self.error = Some(format!("Runtime error: {e}")),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, "Error:");
            ui.colored_label(egui::Color32::RED, error);
            return;
        }

        if self.asm.is_none() {
            ui.label(t!("output.empty"));
            return;
        }

        let asm = self.asm.as_mut().unwrap();
        let mut done = false;
        if self.running {
            let curr_inst = asm.iter_mut().find(|(_, p)| p < &1.0);
            if let Some((inst, progress)) = curr_inst {
                if progress == &0.0 {
                    // advance the interpreter
                    let _ = self.interpreter.as_mut().unwrap().step();
                }
                let progress_increment = match inst {
                    Inst::Add(_, _) => 0.03333,
                    Inst::Sub(_, _) => 0.03333,
                    Inst::Mul(_, _) => 0.01667,
                    Inst::Div(_, _) => 0.00833,
                    Inst::Shl(_, _) => 0.03333,
                    Inst::Shr(_, _) => 0.03333,
                    Inst::Store(_, _) => 0.0667,
                    Inst::Transfer(_, _) => 0.0667,
                    Inst::Result(_) => 0.0667,
                    Inst::Write(_, _) => 0.0033,
                    Inst::Load(_, _) => 0.0033,
                };
                *progress += progress_increment;
                self.total_time += 0.016667; // 60 fps?
            } else {
                done = true;
            }
        }

        ui.horizontal_top(|ui| {
            egui::Grid::new("register_layout")
                .num_columns(2)
                .spacing([30.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    let reg_count = self.hw.as_ref().unwrap().num_registers;
                    for num in 0..reg_count {
                        let reg = u8tochar(num);
                        ui.label(format!("Register {}", reg));
                        ui.label(format!(
                            "{}",
                            self.interpreter
                                .as_ref()
                                .map_or(&0, |i| i.reg_store.get(&reg).unwrap_or(&0))
                        ));
                        ui.end_row();
                    }
                });
            egui::Grid::new("ram_layout")
                .num_columns(2)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    let ram_size = self.hw.as_ref().unwrap().num_cachelines;
                    for num in 0..ram_size {
                        if (num % 2) == 0 && num > 0 {
                            ui.end_row();
                        }
                        ui.label(format!(
                            "{}",
                            self.interpreter.as_ref().map_or(0, |i| i.ram[num])
                        ));
                    }
                    if (ram_size % 2) != 0 {
                        ui.label(" ");
                    }
                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        ui.scope_builder(
            egui::UiBuilder::new().id_salt("interactive_container"),
            |ui| {
                let visuals = ui.style().noninteractive();
                let text_color = visuals.text_color();

                egui::Frame::canvas(ui.style())
                    .fill(visuals.bg_fill.gamma_multiply(0.3))
                    .stroke(visuals.bg_stroke)
                    .inner_margin(ui.spacing().menu_margin)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.add_space(32.0);
                        ui.vertical_centered(|ui| {
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{}",
                                    self.interpreter
                                        .as_ref()
                                        .map_or(" ".into(), |i| i.cur_as_string())
                                ))
                                .color(text_color)
                                .size(32.0),
                            )
                            .selectable(false)
                            .ui(ui);
                        });
                        ui.add_space(32.0);
                    });
            },
        );

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .show(ui, |ui| {
                egui::Grid::new(self.heading.clone())
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .min_col_width(30.0)
                    .show(ui, |ui| {
                        for (inst, progress) in asm {
                            let bar = egui::ProgressBar::new(*progress)
                                .animate(true)
                                .desired_width(30.0)
                                .desired_height(7.5);
                            let v = progress > &mut 0.0;
                            ui.add_visible(v, bar);
                            ui.label(format!("{inst}"));
                            ui.end_row();
                        }
                    });
            });

        if self.running {
            ui.separator();
            ui.label(t!("output.time", t = self.total_time.round()));
        }
        if done {
            ui.separator();
            ui.label(t!("output.result", res = self.program_result.unwrap()));
        }
    }
}

impl crate::gui::Window for AssemblyOutput {
    fn name(&self) -> String {
        self.heading.clone()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(t!(self.name()))
            .id(Id::new(self.name()))
            .open(open)
            .default_height(600.0)
            .show(ctx, |ui| self.ui(ui));
    }
}
