use std::collections::HashMap;

use camino::Utf8Path;
use color_eyre::eyre;
use image::Rgba;
use once_cell::sync::Lazy;

use crate::blocks;
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::TextureFilteringMode;
use crate::helpers::FillPixels;

pub static STAIR_BLOCKS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("blocks.txt").lines().collect::<Vec<_>>()
});

pub fn get_stair_block_textures(texture_filtering_mode: &TextureFilteringMode, block_textures_path: &Utf8Path) -> eyre::Result<HashMap<String, TextureWithBlockState>> {
    Ok(
        blocks::get_block_textures(texture_filtering_mode, block_textures_path, &STAIR_BLOCKS).into_iter()
            .flat_map(|(name, TextureWithBlockState { texture, block_id, .. })| {
                (0..4)
                    .map(|i| {
                        let mut texture = texture.clone();
                        texture.fill_pixels((i % 2) * 8, (i / 2) * 8, 8, 8, Rgba([0, 0, 0, 0]));

                        (format!("{}_stair_{}", &name, i * 90), TextureWithBlockState {
                            texture,
                            block_id: block_id.clone(),
                            block_state_properties: Some(
                                match i {
                                    0 => [("facing".to_string(), "east".to_string()), ("half".to_string(), "bottom".to_string())],
                                    1 => [("facing".to_string(), "west".to_string()), ("half".to_string(), "bottom".to_string())],
                                    2 => [("facing".to_string(), "east".to_string()), ("half".to_string(), "top".to_string())],
                                    _ => [("facing".to_string(), "west".to_string()), ("half".to_string(), "top".to_string())],
                                }.into()
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<HashMap<_, _>>()
    )
}