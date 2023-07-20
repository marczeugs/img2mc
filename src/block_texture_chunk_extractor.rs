use std::collections::HashMap;

use color_eyre::eyre;
use image::{GenericImageView, Rgba, RgbaImage};
use image::imageops::FilterType;

use crate::{blocks, CliArguments};
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::TextureFilteringMode;

pub struct BlockTextureData {
    pub block_textures_and_states: HashMap<String, TextureWithBlockState>,
    pub chunk_average_color_map: HashMap<String, Vec<Vec<Rgba<u8>>>>,
}

pub fn extract(cli_arguments: &CliArguments) -> eyre::Result<BlockTextureData> {
    let texture_filtering_mode = if let Some(block_palette) = &cli_arguments.block_palette {
        TextureFilteringMode::AllowList(block_palette.0.clone())
    } else if cli_arguments.exclude_non_survival_blocks {
        TextureFilteringMode::BlockList(include_str!("non_survival_blocks.txt").lines().map(|s| s.into()).collect::<Vec<_>>())
    } else {
        TextureFilteringMode::BlockList(vec![])
    };

    let mut block_textures_and_states: HashMap<String, TextureWithBlockState> = HashMap::new();
    block_textures_and_states.extend([("air".into(), TextureWithBlockState {
        texture: RgbaImage::new(16, 16).into(),
        block_id: "minecraft:air".into(),
        block_state_properties: None,
    })]);
    block_textures_and_states.extend(blocks::get_normal_block_textures(&texture_filtering_mode, &cli_arguments.block_textures_path)?);
    block_textures_and_states.extend(blocks::get_stair_block_textures(&texture_filtering_mode, &cli_arguments.block_textures_path)?);
    block_textures_and_states.extend(blocks::get_slab_block_textures(&texture_filtering_mode, &cli_arguments.block_textures_path)?);
    block_textures_and_states.extend(blocks::get_rotate_4_way_textures(&texture_filtering_mode, &cli_arguments.block_textures_path)?);
    // special cases: cauldron_side, fence, fence gate, campfire, daylight_detector

    let block_chunk_data = block_textures_and_states.iter()
        .map(|(name, TextureWithBlockState { texture, .. })| {
            let mut chunks_average_color = vec![vec![Rgba([0; 4]); cli_arguments.chunk_resolution]; cli_arguments.chunk_resolution];

            for x in 0..cli_arguments.chunk_resolution {
                for y in 0..cli_arguments.chunk_resolution {
                    let chunk = texture.crop_imm(
                        (x * 16 / cli_arguments.chunk_resolution) as u32,
                        (y * 16 / cli_arguments.chunk_resolution) as u32,
                        (16 / cli_arguments.chunk_resolution) as u32,
                        (16 / cli_arguments.chunk_resolution) as u32
                    );

                    let image_buffer = chunk.resize(1, 1, FilterType::Triangle);

                    chunks_average_color[x][y] = image_buffer.get_pixel(0, 0);
                }
            }

            (name.clone(), chunks_average_color)
        })
        .collect();

    Ok(BlockTextureData {
        block_textures_and_states,
        chunk_average_color_map: block_chunk_data
    })
}