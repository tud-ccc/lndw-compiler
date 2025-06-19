use std::collections::BTreeSet;

use crate::gui::{AssemblyOutput, CodeEditor, EditorAction, InterpreterOptions, Window};
use eframe::egui::{self, FontData, FontFamily, Modifiers, Ui};
use eframe::epaint::text::{FontInsert, InsertFontFamily};

macro_rules! add_sidebar_item {
    ($ui: expr, $open: expr, $item: expr) => {
        let mut is_open = $open.contains(&$item.name());
        $ui.toggle_value(&mut is_open, $item.name());
        set_open(&mut $open, &$item.name(), is_open);
    };
}

macro_rules! add_window {
    ($ctx: expr, $open: expr, $item: expr) => {
        let mut is_open = $open.contains(&$item.name());
        $item.show($ctx, &mut is_open);
        set_open(&mut $open, &$item.name(), is_open);
    };
}

fn set_open(open: &mut BTreeSet<String>, key: &str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

#[derive(Default)]
pub struct LndwApp {
    code_editor: CodeEditor,
    interpreter_options: InterpreterOptions,
    asm_unoptimized: AssemblyOutput,
    asm_optimized: AssemblyOutput,
    result: Option<String>,

    /// List of open windows
    open: BTreeSet<String>,
}

impl LndwApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.5);

        cc.egui_ctx.add_font(FontInsert::new(
            "Iosevka Term Regular",
            FontData::from_static(include_bytes!("../assets/IosevkaTerm-Regular.ttf")),
            vec![InsertFontFamily {
                family: FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));

        let mut res = Self {
            asm_unoptimized: AssemblyOutput::empty("Unoptimized output".to_string()),
            asm_optimized: AssemblyOutput::empty("Optimized output".to_string()),
            ..Self::default()
        };

        set_open(&mut res.open, &res.code_editor.name(), true);

        res
    }
}

impl eframe::App for LndwApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("window_selector")
            .resizable(false)
            .default_width(160.0)
            .min_width(160.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| ui.heading("Tools"));

                ui.separator();

                // window list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        add_sidebar_item!(ui, self.open, self.code_editor);
                        add_sidebar_item!(ui, self.open, self.asm_unoptimized);
                        add_sidebar_item!(ui, self.open, self.asm_optimized);
                        add_sidebar_item!(ui, self.open, self.interpreter_options);

                        ui.toggle_value(&mut false, "Optimizations");

                        ui.separator();
                        if ui.button("Organize windows").clicked() {
                            ui.ctx().memory_mut(|mem| mem.reset_areas());
                        }
                    });
                });
            });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                file_menu_button(ui);
            });
        });

        add_window!(ctx, self.open, self.code_editor);

        // code actions?
        for action in self.code_editor.actions.drain(..) {
            match action {
                EditorAction::Compile => {
                    if let Ok(vars) = self.asm_unoptimized.compile(&self.code_editor.code, false) {
                        self.code_editor.input_variables =
                            vars.iter().map(|s| (s.clone(), String::new())).collect();
                    } else {
                        self.code_editor.input_variables.clear();
                    }

                    if self.code_editor.do_constant_folding {
                        // TODO: consider what to do with vars & any errors.
                        let _ = self.asm_optimized.compile(&self.code_editor.code, true);
                    }

                    set_open(&mut self.open, &self.asm_optimized.name(), true);
                    set_open(&mut self.open, &self.asm_unoptimized.name(), true);
                }
                EditorAction::Run => {
                    set_open(&mut self.open, &self.asm_optimized.name(), true);
                    set_open(&mut self.open, &self.asm_unoptimized.name(), true);
                    self.asm_unoptimized.run(&self.code_editor.input_variables);
                    self.asm_optimized.run(&self.code_editor.input_variables);
                }
                EditorAction::Clear => {
                    self.asm_unoptimized.clear();
                    self.asm_optimized.clear();
                    self.result = None;
                }
            }
        }

        add_window!(ctx, self.open, self.asm_unoptimized);
        add_window!(ctx, self.open, self.asm_optimized);
        add_window!(ctx, self.open, self.interpreter_options);

        egui::CentralPanel::default().show(ctx, |_| {});
    }
}

fn file_menu_button(ui: &mut Ui) {
    let organize_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::O);
    let reset_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::R);

    // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
    // or else they would only be checked if the "File" menu was actually open!

    if ui.input_mut(|i| i.consume_shortcut(&organize_shortcut)) {
        ui.ctx().memory_mut(|mem| mem.reset_areas());
    }

    if ui.input_mut(|i| i.consume_shortcut(&reset_shortcut)) {
        ui.ctx().memory_mut(|mem| *mem = Default::default());
    }

    egui::widgets::global_theme_preference_switch(ui);

    ui.separator();

    ui.menu_button("View", |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        egui::gui_zoom::zoom_menu_buttons(ui);
        ui.weak(format!(
            "Current zoom: {:.0}%",
            100.0 * ui.ctx().zoom_factor()
        ))
        .on_hover_text("The UI zoom level, on top of the operating system's default value");
        ui.separator();

        if ui
            .add(
                egui::Button::new("Organize Windows")
                    .shortcut_text(ui.ctx().format_shortcut(&organize_shortcut)),
            )
            .clicked()
        {
            ui.ctx().memory_mut(|mem| mem.reset_areas());
        }

        if ui
            .add(
                egui::Button::new("Reset egui memory")
                    .shortcut_text(ui.ctx().format_shortcut(&reset_shortcut)),
            )
            .on_hover_text("Forget scroll, positions, sizes etc")
            .clicked()
        {
            ui.ctx().memory_mut(|mem| *mem = Default::default());
        }
    });

    ui.selectable_label(false, "German");
    ui.selectable_label(true, "English");
}
