use eframe::egui;
use egui_extras::image::RetainedImage;
use egui_extras::StripBuilder;
use std::future::Future;
use std::io;
use std::path::PathBuf;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PixeliteApp {
    open_file_path: Option<PathBuf>,
    // Example stuff:
    setting_window: bool,
    input_window: bool,
    output_window: bool,
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    pixel_size: i32,
    color_distortion: i32,

    #[serde(skip)]
    image: Option<RetainedImage>,
}

impl Default for PixeliteApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            open_file_path: None,
            pixel_size: 16,
            color_distortion: 5,
            setting_window: true,
            input_window: true,
            output_window: true,
            image: None,
        }
    }
}

impl PixeliteApp {
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

impl eframe::App for PixeliteApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            open_file_path,
            pixel_size,
            color_distortion,
            setting_window,
            input_window,
            output_window,
            image,
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
                        frame.close();
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
                ui.separator();

                ui.label("Load a  picture:");
                if ui.button("Open").clicked() {
                    *open_file_path = choose_file(frame);
                    let file_bytes = std::fs::read(open_file_path.as_ref().unwrap()).unwrap();

                    self.image =
                        Some(RetainedImage::from_image_bytes("process", &file_bytes).unwrap());
                }

                ui.label("Pixel Size: ");
                ui.horizontal(|ui| {
                    ui.selectable_value(pixel_size, 16, "16 * 16");
                    ui.selectable_value(pixel_size, 32, "32 * 32");
                    ui.selectable_value(pixel_size, 64, "64 * 64");
                    ui.selectable_value(pixel_size, 128, "128 * 128");
                    ui.selectable_value(pixel_size, 256, "256 * 256");
                });
                ui.end_row();

                ui.label("Color distortion: ");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(color_distortion, 0..=10));
                });
                ui.end_row();

                if ui.button("Generate").clicked() {
                    *output_window = true;
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
                egui::Window::new("Input").show(ctx, |ui| {
                    if let Some(image) = &self.image {
                        image.show(ui);
                    }
                    egui::warn_if_debug_build(ui);
                });
            }

            if *output_window {
                egui::Window::new("Output Image").show(ctx, |ui| {
                    ui.label("No output image yet. Click Generate to generate one.");
                    egui::warn_if_debug_build(ui);
                });
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn choose_file(frame: &mut eframe::Frame) -> Option<PathBuf> {
    let task = rfd::AsyncFileDialog::new().pick_file();
    execute(async move {
        let file = task.await;
        return file;
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn choose_file(frame: &mut eframe::Frame) -> Option<PathBuf> {
    rfd::FileDialog::new().pick_file()
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
