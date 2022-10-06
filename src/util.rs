use egui::{color::Color32, Vec2};
use image::DynamicImage;
use kmeans_colors::{get_kmeans, Calculate, Kmeans, MapColor, Sort};
use palette::{FromColor, IntoColor, Lab, Pixel, Srgb};

pub fn calculate_kmeans(image: DynamicImage, k: usize) -> Option<Vec<Color32>> {
    let img_vec = image.into_rgb8().to_vec();

    let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
        .iter()
        .map(|x| x.into_format().into_color())
        .collect();

    let run = 10;
    let max_iter = 20;
    let converge = 1.0;
    let verbose = false;
    let seed = 0;

    let mut result = Kmeans::new();
    for i in 0..run {
        let run_result = get_kmeans(k, max_iter, converge, verbose, &lab, seed + i as u64);
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
    Some(color_palette)
}

pub fn calc_target_size(image: DynamicImage, pixel_size: usize) -> Vec2 {
    let width = image.width() as usize;
    let height = image.height() as usize;

    let target_width = width as f32 / pixel_size as f32;
    let target_height = height as f32 / pixel_size as f32;
    Vec2::new(target_width, target_height)
}
