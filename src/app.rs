/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    setting_window: bool,
    input_window: bool,
    output_window: bool,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    texture: Option<egui::TextureHandle>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            setting_window: true,
            input_window: true,
            output_window: true,
            texture: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            setting_window,
            input_window,
            output_window,
            value,
            texture,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::global_dark_light_mode_switch(ui);
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        todo!()
                    }
                    if ui.button("Save as").clicked() {
                        todo!()
                    }

                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.button("Setting").clicked() {
                        *setting_window = !*setting_window;
                    }
                    if ui.button("Input").clicked() {
                        *input_window = !*input_window;
                    }
                    if ui.button("Output").clicked() {
                        *output_window = !*output_window;
                    }
                })
            });
        });

        if *setting_window {
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                ui.heading("Setting Panel");

                ui.horizontal(|ui| {
                    ui.label("Write something: ");
                    ui.text_edit_singleline(label);
                });

                ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
                if ui.button("Increment").clicked() {
                    *value += 1.0;
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("create by ");
                        ui.hyperlink_to("aizcutei", "https://aizcutei.com");
                        ui.label(".");
                    });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if *input_window {
                egui::Window::new("Input Image").show(ctx, |ui| {
                    ui.label("This is a window");
                    egui::warn_if_debug_build(ui);
                });
            }

            if *output_window {
                egui::Window::new("Output Image").show(ctx, |ui| {
                    ui.label("This is a window");
                    egui::warn_if_debug_build(ui);
                });
            }
        });
    }
}
