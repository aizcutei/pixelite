use egui::{color::Color32, ColorImage, Vec2};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, Rgba};
use kmeans_colors::{get_kmeans, get_kmeans_hamerly, Calculate, Kmeans, MapColor, Sort};
use palette::{FromColor, Hsv, IntoColor, Lab, Pixel, Srgb};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KmeansParams {
    pub k: usize,
    pub run: usize,
    pub max_iter: usize,
    pub converge: f32,
    pub verbose: bool,
    pub seed: u64,
}

pub fn calculate_kmeans(
    image: DynamicImage,
    params: KmeansParams,
) -> Option<(Vec<Color32>, Vec<Lab>)> {
    let img_vec = image.into_rgb8().to_vec();

    let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
        .iter()
        .map(|x| x.into_format().into_color())
        .collect();
    let k = params.k;
    let run = params.run;
    let max_iter = params.max_iter;
    let converge = params.converge;
    let verbose = params.verbose;
    let seed = params.seed;

    let mut result = Kmeans::new();
    for i in 0..run {
        let run_result = get_kmeans_hamerly(k, max_iter, converge, verbose, &lab, seed + i as u64);
        if run_result.score < result.score {
            result = run_result;
        }
    }

    let rgb = &result
        .centroids
        .iter()
        .map(|x| Srgb::from_color(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    let mut color_palette = Vec::<Color32>::new();
    for color in rgb {
        color_palette.push(Color32::from_rgb(color.red, color.green, color.blue));
    }

    /* Get one dominant color
    let buffer = Srgb::map_indices_to_centroids(&rgb, &result.indices);
    let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);

    let dominant_color = Lab::get_dominant_color(&res);

    res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());

    let dominant_color = res.first().unwrap().centroid;
    */
    Some((color_palette, result.centroids))
}

pub fn calc_target_size(image: DynamicImage, pixel_size: usize) -> Option<Vec2> {
    let width = image.width() as usize;
    let height = image.height() as usize;

    if width <= (2 * pixel_size) || height <= (2 * pixel_size) {
        return None;
    }

    let target_width = width as f32 / pixel_size as f32;
    let target_height = height as f32 / pixel_size as f32;
    Some(Vec2::new(target_width.floor(), target_height.floor()))
}

pub fn generate_image(
    image: DynamicImage,
    pixel_size: usize,
    size: Vec2,
    colors: Vec<Lab>,
) -> DynamicImage {
    let img_vec = image.into_rgb8();
    let mut output_img = RgbImage::new(size.x as u32, size.y as u32);

    for i in 0..size.x as usize {
        for j in 0..size.y as usize {
            let mut pixel_sum_r = 0;
            let mut pixel_sum_g = 0;
            let mut pixel_sum_b = 0;
            for k in 0..pixel_size {
                for l in 0..pixel_size {
                    let pixel =
                        img_vec.get_pixel((i * pixel_size + k) as u32, (j * pixel_size + l) as u32);
                    pixel_sum_r += pixel[0] as usize;
                    pixel_sum_g += pixel[1] as usize;
                    pixel_sum_b += pixel[2] as usize;
                }
            }
            let pixel_avg_r = pixel_sum_r / (pixel_size * pixel_size);
            let pixel_avg_g = pixel_sum_g / (pixel_size * pixel_size);
            let pixel_avg_b = pixel_sum_b / (pixel_size * pixel_size);
            let pixel_avg = Rgb([pixel_avg_r as u8, pixel_avg_g as u8, pixel_avg_b as u8]);
            let output_pixel = choose_closest_color(pixel_avg, colors.clone());
            output_img.put_pixel(i as u32, j as u32, output_pixel);
        }
    }
    DynamicImage::ImageRgb8(output_img)
}

fn choose_closest_color(pixel: Rgb<u8>, pixels: Vec<Lab>) -> Rgb<u8> {
    let binding = Srgb::from_raw_slice(&[pixel[0], pixel[1], pixel[2]])
        .iter()
        .map(|x| x.into_format().into_color())
        .collect::<Vec<Lab>>();
    let pixel_lab = binding.first().unwrap();
    let mut closest: Lab = *pixels.first().unwrap();
    let mut min_delta = std::f32::MAX;
    for c in pixels {
        let delta = delta_e(*pixel_lab, c);
        if delta < min_delta {
            min_delta = delta;
            closest = c;
        }
    }

    let closest_rgb = Srgb::from_color(closest).into_format();
    Rgb::from([closest_rgb.red, closest_rgb.green, closest_rgb.blue])
}

fn delta_e(a: Lab, b: Lab) -> f32 {
    (a.a - b.a).powi(2) + (a.b - b.b).powi(2) + (a.l - b.l).powi(2)
}

pub fn dynamic_image_to_color_image(image: DynamicImage) -> ColorImage {
    let img_vec = image.clone().into_rgba8().to_vec();

    ColorImage::from_rgba_unmultiplied(
        [
            image.width().try_into().unwrap(),
            image.height().try_into().unwrap(),
        ],
        &img_vec,
    )
}
pub fn get_pixel(image: DynamicImage, x: u32, y: u32) -> Rgba<u8> {
    let img_vec = image.into_rgba8();
    let pixel = img_vec.get_pixel(x, y);
    *pixel
}

pub fn convolver(image: DynamicImage, matrix: [[i32; 3]; 3], divisor: i32) -> DynamicImage {
    let w = image.width();
    let h = image.height();
    let matrix_size = 3;

    let img_copy = image;
    let mut result = RgbImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let mstarty = y - 1;
            let mstartx = x - 1;

            let mut accum_r = 0;
            let mut accum_g = 0;
            let mut accum_b = 0;

            for (m_index_y, mut img_y) in (0..).zip(mstarty..mstarty + matrix_size) {
                if img_y < 0 {
                    img_y = 0;
                } else if img_y > h - 1 {
                    img_y = h - 1;
                }

                for (m_index_x, mut img_x) in (0..).zip(mstartx..mstartx + matrix_size) {
                    if img_x < 0 {
                        img_x = 0;
                    } else if img_x > w - 1 {
                        img_x = w - 1;
                    }

                    let pixel = get_pixel(img_copy.clone(), img_x as u32, img_y as u32);
                    accum_r += pixel[0] as i32 * matrix[m_index_y][m_index_x];
                    accum_g += pixel[1] as i32 * matrix[m_index_y][m_index_x];
                    accum_b += pixel[2] as i32 * matrix[m_index_y][m_index_x];
                }
            }

            if divisor != 1 {
                accum_r /= divisor;
                accum_g /= divisor;
                accum_b /= divisor;
            }

            if accum_r < 0 {
                accum_r = 0;
            } else if accum_r > 255 {
                accum_r = 255;
            }

            if accum_g < 0 {
                accum_g = 0;
            } else if accum_g > 255 {
                accum_g = 255;
            }

            if accum_b < 0 {
                accum_b = 0;
            } else if accum_b > 255 {
                accum_b = 255;
            }

            let result_pixel = Rgb([accum_r as u8, accum_g as u8, accum_b as u8]);
            result.put_pixel(x, y, result_pixel);
        }
    }

    DynamicImage::ImageRgb8(result)
}

pub fn sharpen_filter(image: DynamicImage) -> DynamicImage {
    let matrix = [[0, -1, 0], [-1, 5, -1], [0, -1, 0]];
    convolver(image, matrix, 1)
}
