use eframe::egui;
use egui_extras::image::RetainedImage;
#[cfg(target_arch = "wasm32")]
use futures::Future;
use image::{codecs::png::PngEncoder, DynamicImage, ImageEncoder};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::util::{self, dynamic_image_to_color_image, KmeansParams};

const DEBUG: bool = false;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PixeliteApp {
    #[cfg(not(target_arch = "wasm32"))]
    open_file_path: Option<PathBuf>,

    #[cfg(target_arch = "wasm32")]
    open_file_path: Option<String>,

    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,
    #[serde(skip)]
    dropped_file: Option<egui::DroppedFile>,

    information: String,
    dbg_information: String,

    setting_window: bool,
    input_window: bool,
    output_window: bool,
    info_window: bool,
    color_palette_window: bool,
    is_loading: bool,

    pixel_size: usize,

    raw_input: Option<Vec<u8>>,
    raw_output: Option<Vec<u8>>,

    #[serde(skip)]
    img_dyn: Option<DynamicImage>,

    #[serde(skip)]
    output_img_dyn: Option<DynamicImage>,

    #[serde(skip)]
    color_palette: Option<Vec<egui::Color32>>,

    #[serde(skip)]
    image: Option<RetainedImage>,

    #[serde(skip)]
    output_image: Option<RetainedImage>,

    #[serde(skip)]
    kmeans_params: KmeansParams,
}

impl Default for PixeliteApp {
    fn default() -> Self {
        Self {
            open_file_path: None,
            dropped_files: Default::default(),
            dropped_file: None,
            information: String::new(),
            dbg_information: String::new(),
            setting_window: true,
            input_window: true,
            output_window: true,
            info_window: false,
            color_palette_window: false,
            is_loading: false,
            pixel_size: 16,
            raw_input: None,
            raw_output: None,
            img_dyn: None,
            output_img_dyn: None,
            color_palette: None,
            image: None,
            output_image: None,
            kmeans_params: KmeansParams {
                k: 5,
                run: 10,
                max_iter: 20,
                converge: 1.0,
                verbose: false,
                seed: 0,
            },
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
            dropped_files,
            dropped_file,
            open_file_path,
            information,
            dbg_information,
            pixel_size,
            setting_window,
            input_window,
            output_window,
            info_window,
            color_palette_window,
            is_loading,
            raw_input,
            raw_output,
            img_dyn,
            output_img_dyn,
            color_palette,
            image,
            output_image,
            kmeans_params,
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
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Open").clicked() {
                        self.open_file_path = choose_file(frame);
                        let file_bytes = std::fs::read(self.open_file_path.as_ref().unwrap())
                            .unwrap_or_default();
                        self.raw_input = Some(file_bytes.clone());
                        self.image =
                            Some(RetainedImage::from_image_bytes("process", &file_bytes).unwrap());
                    }
                    if ui.button("Save as").clicked() {
                        if self.raw_output.is_some() {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let save = rfd::FileDialog::new()
                                    .set_file_name("output.png")
                                    .save_file();
                                if let Some(path) = save {
                                    let mut file = std::fs::File::create(path).unwrap();
                                    let mut encoder = PngEncoder::new(&mut file);
                                    encoder
                                        .write_image(
                                            self.raw_output.as_ref().unwrap(),
                                            self.output_img_dyn.as_ref().unwrap().width(),
                                            self.output_img_dyn.as_ref().unwrap().height(),
                                            image::ColorType::Rgb8,
                                        )
                                        .unwrap();
                                }
                            }
                        } else {
                            *information = "No output to save".to_string();
                        }
                    }

                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.button("Setting").clicked() {
                        self.setting_window = !self.setting_window;
                    }
                    if ui.button("Input").clicked() {
                        self.input_window = !self.input_window;
                    }
                    if ui.button("Output").clicked() {
                        self.output_window = !self.output_window;
                    }
                    if ui.button("Color").clicked() {
                        self.color_palette_window = !self.color_palette_window;
                    }
                });
                if ui.button("Rearrenge").clicked() {
                    ui.ctx().memory().reset_areas();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            if self.setting_window {
                egui::Window::new("Setting").show(ctx, |ui| {
                    ui.heading("Setting Panel");
                    ui.separator();

                    ui.label("Load a  picture:");
                    ui.horizontal(|ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if ui.button("Open").clicked() {
                                {
                                    self.open_file_path = choose_file(frame);
                                    let file_bytes =
                                        std::fs::read(self.open_file_path.as_ref().unwrap())
                                            .unwrap();
                                    self.raw_input = Some(file_bytes.clone());
                                    self.img_dyn =
                                        Some(image::load_from_memory(&file_bytes).unwrap());
                                    self.image = Some(
                                        RetainedImage::from_image_bytes("process", &file_bytes)
                                            .unwrap(),
                                    );
                                }
                            }
                            if let Some(path) = &self.open_file_path {
                                ui.label(format!("Loaded: {}", path.display()));
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            if ui.button("Drag a file here to open").clicked() {}
                            if let Some(path) = &open_file_path {
                                ui.label(format!("Loaded: {}", path));
                            }
                        }
                    });

                    ui.label("Pixel Size: ");
                    ui.horizontal(|ui| {
                        ui.selectable_value(pixel_size, 4, "4");
                        ui.selectable_value(pixel_size, 8, "8");
                        ui.selectable_value(pixel_size, 16, "16");
                        ui.selectable_value(pixel_size, 32, "32");
                        ui.selectable_value(pixel_size, 64, "64");
                        ui.selectable_value(pixel_size, 128, "128");

                        ui.add(
                            egui::DragValue::new(pixel_size)
                                .speed(1.0)
                                .clamp_range(1..=2048),
                        );
                    });
                    ui.end_row();

                    ui.label("Color distortion: ");
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut kmeans_params.k, 2..=20));
                    });
                    ui.end_row();

                    ui.collapsing("Advanced", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Runs: ");
                            ui.add(egui::DragValue::new(&mut kmeans_params.run).speed(1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Iterations: ");
                            ui.add(egui::DragValue::new(&mut kmeans_params.max_iter).speed(1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Converge");
                            ui.add(egui::DragValue::new(&mut kmeans_params.converge).speed(1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Seed: ");
                            ui.add(egui::DragValue::new(&mut kmeans_params.seed).speed(1.0));
                        });
                    });

                    if ui.button("Generate").clicked() {
                        if self.img_dyn.is_some() {
                            let (rgb_palette, lab_palette) = util::calculate_kmeans(
                                self.img_dyn.as_ref().unwrap().clone(),
                                *kmeans_params,
                            )
                            .unwrap();
                            self.color_palette = Some(rgb_palette);

                            let output_size = util::calc_target_size(
                                self.img_dyn.as_ref().unwrap().clone(),
                                *pixel_size,
                            );
                            if output_size.is_none() {
                                self.information = "Pixel size is too large".to_string();
                            } else {
                                //let sharpened_img =
                                //    util::sharpen_filter(self.img_dyn.as_ref().unwrap().clone());

                                let output_img = util::generate_image(
                                    self.img_dyn.as_ref().unwrap().clone(),
                                    //sharpened_img,
                                    *pixel_size,
                                    output_size.unwrap(),
                                    lab_palette,
                                );
                                self.output_img_dyn = Some(output_img.clone());
                                self.raw_output = Some(output_img.to_rgb8().to_vec());

                                self.output_image = Some(RetainedImage::from_color_image(
                                    "output",
                                    dynamic_image_to_color_image(output_img),
                                ));
                            }

                            self.color_palette_window = true;
                            self.output_window = true;
                        } else {
                            self.information = "Please load a picture first!".to_string();
                        }
                    }

                    if self.is_loading {
                        ui.label("Loading...");
                        ui.add(egui::Spinner::new());
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

            if self.input_window {
                egui::Window::new("Input Image")
                    .vscroll(true)
                    .hscroll(true)
                    .resizable(true)
                    .show(ctx, |ui| {
                        if let Some(image) = &self.image {
                            #[cfg(not(target_arch = "wasm32"))]
                            let window_size = frame.info().window_info.size;

                            #[cfg(target_arch = "wasm32")]
                            let window_size = egui::vec2(1080.0, 720.0);
                            let image_size = image.size_vec2();
                            if image_size.x / window_size.x > image_size.y / window_size.y {
                                let scale = window_size.x / (2.0 * image_size.x);
                                image.show_scaled(ui, scale);
                            } else {
                                let scale = window_size.y / (2.0 * image_size.y);
                                image.show_scaled(ui, scale);
                            }

                            self.is_loading = false;
                        } else {
                            ui.label("No image loaded, drag in a image to start.");
                        }
                        egui::warn_if_debug_build(ui);
                    });
            }

            if self.output_window {
                egui::Window::new("Output Image")
                    .vscroll(true)
                    .hscroll(true)
                    .resizable(true)
                    .show(ctx, |ui| {
                        if let Some(image) = &self.output_image {
                            #[cfg(not(target_arch = "wasm32"))]
                            let window_size = frame.info().window_info.size;

                            #[cfg(target_arch = "wasm32")]
                            let window_size = egui::vec2(1080.0, 720.0);
                            let image_size = image.size_vec2();
                            if image_size.x / window_size.x > image_size.y / window_size.y {
                                let scale = window_size.x / (4.0 * image_size.x);
                                image.show_scaled(ui, scale);
                            } else {
                                let scale = window_size.y / (4.0 * image_size.y);
                                image.show_scaled(ui, scale);
                            }
                        } else {
                            ui.label("No output image yet. Click Generate to generate one.");
                        }
                        egui::warn_if_debug_build(ui);
                    });
            }

            if self.info_window {}

            if self.color_palette_window {
                egui::Window::new("Color Palette").show(ctx, |ui| {
                    if self.color_palette.is_none() {
                        ui.label("No color palette yet. Click Generate to generate one.");
                    } else {
                        ui.horizontal(|ui| {
                            for color in self.color_palette.as_ref().unwrap() {
                                let mut c = *color;
                                ui.color_edit_button_srgba(&mut c);
                            }
                        });
                    }

                    egui::warn_if_debug_build(ui);
                });
            }

            // Debug Window
            if DEBUG {
                egui::Window::new("debug").show(ctx, |ui| {
                    ui.group(|ui| {
                        ui.label(self.dbg_information.clone());
                    });
                });
            }

            preview_files_being_dropped(ctx);
        });

        //Drag & Drop related
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
            if !self.dropped_files.is_empty() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let file_copy = self.dropped_files.last().cloned();
                    let last_file_path = file_copy.unwrap().path;
                    let last_file_name = last_file_path
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_owned()
                        .to_str()
                        .unwrap()
                        .to_owned();

                    if last_file_name.ends_with("png")
                        || last_file_name.ends_with("jpg")
                        || last_file_name.ends_with("jpeg")
                        || last_file_name.ends_with("webp")
                    {
                        self.dropped_file = Some(self.dropped_files.last().cloned().unwrap());
                    } else {
                        self.info_window = true;
                        self.information = "Please drop a picture file.".to_string();
                        self.dropped_files.clear();
                        self.dropped_file = None;
                        self.image = None;
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let last_file = self.dropped_files.last().cloned().unwrap();
                    let last_file_name = last_file.name;

                    if last_file_name.ends_with("png")
                        || last_file_name.ends_with("jpg")
                        || last_file_name.ends_with("jpeg")
                        || last_file_name.ends_with("webp")
                    {
                        self.dropped_file = Some(self.dropped_files.last().cloned().unwrap());
                    } else {
                        self.info_window = true;
                        self.information = "Please drop a picture file.".to_string();
                        self.dropped_files.clear();
                        self.dropped_file = None;
                        self.image = None;
                    }
                }
            }

            if self.dropped_file.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let file_copy = self.dropped_files.last().cloned();
                    self.open_file_path = file_copy.unwrap().path;
                    if let Some(path) = self.open_file_path.as_ref() {
                        let file_bytes = std::fs::read(path).unwrap();
                        self.raw_input = Some(file_bytes.clone());
                        self.img_dyn = Some(image::load_from_memory(&file_bytes).unwrap());
                        self.image =
                            Some(RetainedImage::from_image_bytes("process", &file_bytes).unwrap());
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    self.open_file_path = Some(self.dropped_file.as_ref().unwrap().name.clone());
                    let file_copy = self.dropped_files.last().cloned();
                    let raw_bytes = file_copy.unwrap().bytes.unwrap().clone();
                    self.raw_input = Some(raw_bytes.to_vec().clone());
                    self.img_dyn = Some(image::load_from_memory(&raw_bytes).unwrap());
                    self.image =
                        Some(RetainedImage::from_image_bytes("process", &raw_bytes).unwrap());
                }
            }
        }
    }
}

fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input().raw.hovered_files.is_empty() {
        let mut text = "Choosed files:\n".to_owned();
        for file in &ctx.input().raw.hovered_files {
            if let Some(path) = &file.path {
                write!(text, "\n{}", path.display()).ok();
            } else if !file.mime.is_empty() {
                write!(text, "\n{}", file.mime).ok();
            } else {
                text += "\n???";
            }
        }

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.input().screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn choose_file(_frame: &mut eframe::Frame) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(
            "image",
            &["jpg", "jpeg", "png", "bmp", "ico", "tiff", "webp"],
        )
        .pick_file()
}

/*
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
*/

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(target_arch = "wasm32")]
fn choose_file(frame: &mut eframe::Frame) {
    let task = rfd::AsyncFileDialog::new().pick_file();

    execute(async move {
        let file = task.await;
        if let Some(file) = file {
            let f = file.read().await;
        }
    });
}
