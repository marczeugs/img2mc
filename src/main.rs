use std::{fs, io, thread};
use std::cmp::Ordering;
use std::io::Write;
use std::sync::{Arc, atomic};
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

use color_eyre::eyre;
use color_eyre::eyre::eyre;
use gumdrop::Options;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgba, RgbaImage};
use image::imageops::FilterType;
use lab::Lab;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;

use crate::block_texture_chunk_extractor::BlockTextureData;
use crate::cli_arguments::CliArguments;

mod blocks;
pub mod block_texture_chunk_extractor;
pub mod cli_arguments;
pub mod helpers;
pub mod litematic_generator;


fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli_arguments = CliArguments::parse_args_default_or_exit();


    let BlockTextureData { block_textures_and_states, chunk_average_color_map } = block_texture_chunk_extractor::extract(&cli_arguments)?;
    tracing::info!("Loaded {} texture(s) into {} chunks.", chunk_average_color_map.len(), chunk_average_color_map.len() * cli_arguments.chunk_resolution * cli_arguments.chunk_resolution);

    let (source_image, block_width) = get_source_image(&cli_arguments)?;


    let mut output_blocks: Vec<Vec<String>> = vec![vec![String::new(); cli_arguments.block_height]; block_width];

    let mut error_values = vec![
        vec![
            Rgba([0_isize; 4]);
            source_image.height() as usize / cli_arguments.chunk_resolution
        ];
        source_image.width() as usize / cli_arguments.chunk_resolution
    ];

    let dithering_matrix = cli_arguments.dithering_matrix.to_matrix();

    let dithering_center_x = dithering_matrix[0].iter()
        .enumerate()
        .take_while(|(_, &value)| value == 0)
        .last()
        .ok_or(eyre!("Invalid dithering matrix"))?
        .0;

    let dithering_total_weight: usize = dithering_matrix.iter().flat_map(|n| n).sum();


    tracing::info!("Processing chunks...");

    let chunk_width = source_image.width() as usize / cli_arguments.chunk_resolution;
    let chunk_height = source_image.width() as usize / cli_arguments.chunk_resolution;

    let progress_counter = Arc::new(AtomicUsize::new(0));

    print!("[{: <70}]", "");

    let inner_progress_counter = progress_counter.clone();

    let progress_bar_updater_thread = thread::spawn(move || {
        while inner_progress_counter.load(atomic::Ordering::Relaxed) != chunk_width * chunk_height {
            print!(
                "\r[{: <70}] {:.2}% ",
                "#".repeat((inner_progress_counter.load(atomic::Ordering::Relaxed) as f32 / (chunk_width * chunk_height) as f32 * 70.0).round() as usize),
                inner_progress_counter.load(atomic::Ordering::Relaxed) as f32 / (chunk_width * chunk_height) as f32 * 100.0
            );
            io::stdout().flush().unwrap();

            thread::sleep(Duration::from_millis(100));
        }
    });

    for chunk_y in 0..chunk_width {
        for chunk_x in 0..chunk_height {
                let mut error_by_texture = chunk_average_color_map.par_iter()
                .map(|(texture_name, texture_color_map)| {
                    // Calculate the error between every chunk in the source image and the block texture
                    let mut texture_error = 0.0;

                    let mut transparent_pixels_present = false;

                    // Check if transparent pixels are present to decide which error algorithm should be used later
                    'outer: for x_within_chunk in 0..cli_arguments.chunk_resolution {
                        for y_within_chunk in 0..cli_arguments.chunk_resolution {
                            let pixel_rgba_data = source_image.get_pixel(
                                (chunk_x * cli_arguments.chunk_resolution + x_within_chunk) as u32,
                                (chunk_y * cli_arguments.chunk_resolution + y_within_chunk) as u32
                            );

                            if pixel_rgba_data[3] != 255 || texture_color_map[x_within_chunk][y_within_chunk][3] != 255 {
                                transparent_pixels_present = true;
                                break 'outer;
                            }
                        }
                    }

                    for x_within_chunk in 0..cli_arguments.chunk_resolution {
                        for y_within_chunk in 0..cli_arguments.chunk_resolution {
                            // Add residential quantization error to the current chunk
                            let pixel_rgba_data = source_image.get_pixel(
                                (chunk_x * cli_arguments.chunk_resolution + x_within_chunk) as u32,
                                (chunk_y * cli_arguments.chunk_resolution + y_within_chunk) as u32
                            ).map_with_index(|channel, index| (channel as isize + error_values[chunk_x][chunk_y][index]).min(255).max(0) as u8);

                            // Calculate how close the chunk of the current texture is to the source image
                            texture_error += if !transparent_pixels_present {
                                delta_e::DE2000::new(
                                    texture_color_map[x_within_chunk][y_within_chunk].to_lab(),
                                    pixel_rgba_data.to_lab()
                                )
                            } else {
                                euclidean_distance(texture_color_map[x_within_chunk][y_within_chunk], pixel_rgba_data) as f32
                            };
                        }
                    }

                    (texture_name, texture_error)
                })
                .collect::<Vec<_>>();

            // Select texture with lowest error
            error_by_texture.sort_by(|(_, error_1), (_, error_2)| error_1.partial_cmp(error_2).unwrap_or(Ordering::Equal));

            let (lowest_error_texture, _) = error_by_texture[0];
            output_blocks[chunk_x][chunk_y] = lowest_error_texture.clone();

            // Calculating residual quantization error
            let rgba_by_coords_in_block_texture = chunk_average_color_map[lowest_error_texture].iter()
                .enumerate()
                .flat_map(|(x, row)| row.iter()
                    .enumerate()
                    .map(move |(y, rgba_value)| ((x, y), rgba_value))
                )
                .collect::<Vec<_>>();

            let residual_quantization_error = rgba_by_coords_in_block_texture
                .iter()
                .map(|((x, y), block_texture_rgba)| {
                    let pixel_rgba_data = source_image.get_pixel(
                        (chunk_x * cli_arguments.chunk_resolution + x) as u32,
                        (chunk_y * cli_arguments.chunk_resolution + y) as u32
                    ).map_with_index(|channel, index| channel as isize + error_values[*x][*y][index]);

                    block_texture_rgba.map_with_index(|channel, index| pixel_rgba_data[index] - channel as isize)
                })
                .fold(Rgba([0_isize, 0, 0, 0]), |accumulator, next| {
                    accumulator.map_with_index(|channel, index| channel + next[index])
                });

            let residual_quantization_error = residual_quantization_error.map(|channel| channel / (cli_arguments.chunk_resolution * cli_arguments.chunk_resolution) as isize);


            for chunk_y_offset in 0..dithering_matrix.len() {
                for chunk_x_offset in 0..dithering_matrix[0].len() {
                    let dithering_chunk_x = chunk_x as isize + chunk_x_offset as isize - dithering_center_x as isize;
                    let dithering_chunk_y = chunk_y as isize + chunk_y_offset as isize;

                    if dithering_chunk_x >= 0 && dithering_chunk_x < error_values.len() as isize && dithering_chunk_y >= 0 && dithering_chunk_y < error_values[0].len() as isize {
                        let dithering_weight = dithering_matrix[chunk_y_offset][chunk_x_offset] as f32;

                        error_values[dithering_chunk_x as usize][dithering_chunk_y as usize] = error_values[dithering_chunk_x as usize][dithering_chunk_y as usize]
                            .map_with_index(|channel, index| channel + (residual_quantization_error[index] as f32 * (dithering_weight / dithering_total_weight as f32)) as isize);
                    }
                }
            }

            progress_counter.fetch_add(1, atomic::Ordering::Relaxed);
        }
    }

    progress_bar_updater_thread.join().unwrap();
    print!("{: <80}\r", "\r");
    io::stdout().flush()?;


    match cli_arguments.output_path.extension().ok_or(eyre!("Output path does not have a file extension."))? {
        "litematic" | "schematic" => {
            fs::write(&cli_arguments.output_path, litematic_generator::make_bytes(block_width, &cli_arguments, &output_blocks, &block_textures_and_states)?)?;
        }
        _ => {
            let mut output_image = RgbaImage::new((block_width * 16) as u32, (cli_arguments.block_height * 16) as u32);

            for x in 0..block_width {
                for y in 0..cli_arguments.block_height {
                    output_image.copy_from(&block_textures_and_states[&output_blocks[x][y]].texture, (x * 16) as u32, (y * 16) as u32)?;
                }
            }

            output_image.save(&cli_arguments.output_path)?;
        }
    }

    tracing::info!("Saved result to '{}'.", cli_arguments.output_path);


    Ok(())
}

fn get_source_image(cli_arguments: &CliArguments) -> eyre::Result<(DynamicImage, usize)> {
    let source_image = if cli_arguments.input_image_path.starts_with("http") {
        tracing::info!("Loading image with GET request from '{}'...", cli_arguments.input_image_path);

        let image_buffer = reqwest::blocking::get(&cli_arguments.input_image_path)?.bytes()?;
        image::load_from_memory(&image_buffer)?
    } else {
        tracing::info!("Loading image from local path '{}'...", cli_arguments.input_image_path);

        image::open(&cli_arguments.input_image_path)?
    };

    let block_width = cli_arguments.block_width.unwrap_or((cli_arguments.block_height as f32 / source_image.height() as f32 * source_image.width() as f32) as usize);

    let source_image = source_image.resize_exact(
        (block_width * cli_arguments.chunk_resolution) as u32,
        (cli_arguments.block_height * cli_arguments.chunk_resolution) as u32,
        FilterType::Lanczos3
    );

    Ok((source_image, block_width))
}

fn euclidean_distance(rgba_1: Rgba<u8>, rgba_2: Rgba<u8>) -> usize {
    (
        (rgba_1[0] as isize - rgba_2[0] as isize).pow(2)
        + (rgba_1[1] as isize - rgba_2[1] as isize).pow(2)
        + (rgba_1[2] as isize - rgba_2[2] as isize).pow(2)
        + ((rgba_1[3] as isize - rgba_2[3] as isize) * 1).pow(2)
    ) as usize
}

trait MapWithIndex<T> {
    fn map_with_index<F: Fn(T, usize) -> U, U>(&self, f: F) -> Rgba<U>;
}

impl <T: Copy> MapWithIndex<T> for Rgba<T> {
    fn map_with_index<F: Fn(T, usize) -> U, U>(&self, f: F) -> Rgba<U> {
        Rgba([
            f(self.0[0], 0),
            f(self.0[1], 1),
            f(self.0[2], 2),
            f(self.0[3], 3),
        ])
    }
}

trait ToLab {
    fn to_lab(&self) -> Lab;
}

impl ToLab for Rgba<u8> {
    fn to_lab(&self) -> Lab {
        *lab::rgb_bytes_to_labs(&self.0[0..3]).first().unwrap()
    }
}