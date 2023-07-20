use std::collections::HashMap;

use camino::Utf8Path;
use color_eyre::eyre;
use image::imageops;
use once_cell::sync::Lazy;

use crate::blocks;
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::TextureFilteringMode;

pub static ROTATE_4_WAY_BLOCKS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("blocks.txt").lines().collect::<Vec<_>>()
});

pub fn get_rotate_4_way_textures(texture_filtering_mode: &TextureFilteringMode, block_textures_path: &Utf8Path) -> eyre::Result<HashMap<String, TextureWithBlockState>> {
    Ok(
        blocks::get_block_textures(texture_filtering_mode, block_textures_path, &ROTATE_4_WAY_BLOCKS).into_iter()
            .flat_map(|(name, TextureWithBlockState { texture, block_id, .. })| {
                (0..4)
                    .map(|i| {
                        (format!("{}_{}", &name, i * 90), TextureWithBlockState {
                            texture: {
                                (0..i).fold(texture.clone(), |texture, _| imageops::rotate90(&texture).into())
                            },
                            block_id: block_id.clone(),
                            block_state_properties: Some(
                                match i {
                                    0 => [("facing".to_string(), "east".to_string())],
                                    1 => [("facing".to_string(), "north".to_string())],
                                    2 => [("facing".to_string(), "west".to_string())],
                                    _ => [("facing".to_string(), "south".to_string())],
                                }.into()
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<HashMap<_, _>>()
    )
}
