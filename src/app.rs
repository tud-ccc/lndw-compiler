use std::collections::BTreeSet;

use crate::compiler::CompileOptions;
use crate::gui::{AssemblyOutput, CodeEditor, EditorAction, Examples, InterpreterOptions, Window};
use eframe::Frame;
use eframe::egui::{self, Context, FontData, FontFamily, Modifiers, Ui, ViewportCommand};
use eframe::epaint::text::{FontInsert, InsertFontFamily};
use egui::{Align, Button, CentralPanel, Key, KeyboardShortcut, Layout, MenuBar, Panel, ScrollArea, TextWrapMode};
use rust_i18n::t;

macro_rules! add_sidebar_item {
    ($ui: expr, $open: expr, $item: expr) => {
        let mut is_open = $open.contains(&$item.name());
        $ui.toggle_value(&mut is_open, t!($item.name()));
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
    examples: Examples,
    result: Option<String>,
    language: String,
    title_modal_open: bool,
    working_title: String,

    /// List of open windows
    open: BTreeSet<String>,
}

impl LndwApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If we have previously edited the app's name/title, try to load it here
        let name = cc
            .egui_ctx
            .memory_mut(|mem| mem.data.get_persisted::<String>(crate::APP_NAME.into()));
        if let Some(app_name) = name {
            cc.egui_ctx.send_viewport_cmd(ViewportCommand::Title(app_name))
        }

        cc.egui_ctx.set_zoom_factor(1.5);

        cc.egui_ctx.add_font(FontInsert::new(
            "Iosevka Term Regular",
            FontData::from_static(include_bytes!("../assets/IosevkaTerm-Regular.ttf")),
            vec![InsertFontFamily {
                family: FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));

        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut res = Self {
            asm_unoptimized: AssemblyOutput::empty("output.unopt".to_string()),
            asm_optimized: AssemblyOutput::empty("output.opt".to_string()),
            language: "en".to_string(),
            examples: Examples::preloaded(),
            ..Self::default()
        };

        set_open(&mut res.open, &res.code_editor.name(), true);

        res
    }
}

impl eframe::App for LndwApp {
    fn logic(&mut self, _ctx: &Context, _frame: &mut Frame) {
        self.code_editor.disable_run = self.asm_unoptimized.is_running() || self.asm_optimized.is_running();

        // code actions?
        for action in self.code_editor.actions.drain(..) {
            match action {
                EditorAction::Compile => {
                    if let Ok(vars) = self.asm_unoptimized.compile(
                        &self.code_editor.code,
                        CompileOptions::default(),
                        self.interpreter_options,
                    ) {
                        self.code_editor.input_variables = vars.iter().map(|s| (s.clone(), String::new())).collect();
                    } else {
                        self.code_editor.input_variables.clear();
                    }

                    if self.code_editor.compile_options.any() {
                        // TODO: consider what to do with vars & any errors.
                        let _ = self.asm_optimized.compile(
                            &self.code_editor.code,
                            self.code_editor.compile_options,
                            self.interpreter_options,
                        );

                        set_open(&mut self.open, &self.asm_optimized.name(), true);
                    }

                    set_open(&mut self.open, &self.asm_unoptimized.name(), true);
                }
                EditorAction::Run(stepwise) => {
                    set_open(&mut self.open, &self.asm_unoptimized.name(), true);
                    self.asm_unoptimized.run(&self.code_editor.input_variables, stepwise);
                    if self.code_editor.compile_options.any() {
                        set_open(&mut self.open, &self.asm_optimized.name(), true);
                        self.asm_optimized.run(&self.code_editor.input_variables, stepwise);
                    }
                }
                EditorAction::Clear => {
                    self.asm_unoptimized.clear();
                    self.asm_optimized.clear();
                    self.result = None;
                }
            }
        }

        if let Some(choice) = self.examples.chosen {
            self.code_editor.input_variables.clear();
            self.code_editor.code = self.examples.examples[choice].input.into();
            self.code_editor.compile_options = self.examples.examples[choice].options;

            self.examples.chosen = None;
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        Panel::right("window_selector")
            .resizable(false)
            .default_size(160.0)
            .min_size(160.0)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| ui.heading(t!("app.tools")));

                ui.separator();

                // window list
                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        add_sidebar_item!(ui, self.open, self.code_editor);
                        add_sidebar_item!(ui, self.open, self.asm_unoptimized);
                        add_sidebar_item!(ui, self.open, self.asm_optimized);
                        add_sidebar_item!(ui, self.open, self.interpreter_options);
                        add_sidebar_item!(ui, self.open, self.examples);

                        ui.separator();
                        if ui.button(t!("app.organize")).clicked() {
                            ui.ctx().memory_mut(|mem| mem.reset_areas());
                        }

                        // Spacing hack: image is square, so take available width and use it to
                        // subtract from the available height, which we use as filler with add_space
                        let img_width_height = ui.available_width();
                        ui.add_space(ui.available_height() - img_width_height);
                        ui.add(egui::Image::new(egui::include_image!("../assets/icon.png")));
                    });
                });
            });

        Panel::top("menu_bar").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                file_menu_button(
                    ui,
                    &mut self.language,
                    &mut self.title_modal_open,
                    &mut self.working_title,
                );
            });
        });

        add_window!(ui, self.open, self.code_editor);
        add_window!(ui, self.open, self.asm_unoptimized);
        add_window!(ui, self.open, self.asm_optimized);
        add_window!(ui, self.open, self.interpreter_options);
        add_window!(ui, self.open, self.examples);

        CentralPanel::default().show_inside(ui, |_| {});
    }
}

fn file_menu_button(ui: &mut Ui, lang: &mut String, title_modal_open: &mut bool, working_title: &mut String) {
    let organize_shortcut = KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::O);
    let reset_shortcut = KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::R);

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
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

        egui::gui_zoom::zoom_menu_buttons(ui);
        ui.weak(format!("Current zoom: {:.0}%", 100.0 * ui.ctx().zoom_factor()))
            .on_hover_text("The UI zoom level, on top of the operating system's default value");
        ui.separator();

        let organize_button =
            ui.add(Button::new(t!("app.organize")).shortcut_text(ui.ctx().format_shortcut(&organize_shortcut)));
        if organize_button.clicked() {
            ui.ctx().memory_mut(|mem| mem.reset_areas());
        }

        let reset_button = ui
            .add(Button::new(t!("app.reset")).shortcut_text(ui.ctx().format_shortcut(&reset_shortcut)))
            .on_hover_text(t!("app.forget"));
        if reset_button.clicked() {
            ui.ctx().memory_mut(|mem| *mem = Default::default());
        }

        if ui.button(t!("app.change_title")).clicked() {
            *title_modal_open = true;
        }
    });

    if ui.selectable_label(lang.as_str() == "de", t!("app.german")).clicked() {
        rust_i18n::set_locale("de");
        *lang = "de".to_string();
    }
    if ui.selectable_label(lang.as_str() == "en", t!("app.english")).clicked() {
        rust_i18n::set_locale("en");
        *lang = "en".to_string();
    }

    let mut should_update_title = false;
    if *title_modal_open {
        let modal = egui::Modal::new("title modal".into()).show(ui.ctx(), |ui| {
            ui.set_width(300.0);
            ui.heading(t!("app.change_title"));

            ui.add_space(24.0);

            let r = ui.text_edit_singleline(working_title);

            ui.add_space(24.0);

            if ui.button("Save").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))) {
                ui.ctx().memory_mut(|mem| {
                    mem.data.insert_persisted(crate::APP_NAME.into(), working_title.clone());
                });
                *title_modal_open = false;
                should_update_title = true;
            }
        });

        if modal.should_close() {
            *title_modal_open = false;
        }
    }

    // We have to defer until here to prevent deadlock when reading ctx inside Modal::show
    if should_update_title {
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Title(working_title.clone()));
    }
}
