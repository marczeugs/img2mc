use std::collections::HashMap;

use camino::Utf8Path;
use color_eyre::eyre::eyre;
use image::DynamicImage;

pub use normal_blocks::get_normal_block_textures;

use crate::cli_arguments::TextureFilteringMode;

pub use rotate_4_way_blocks::get_rotate_4_way_textures;
pub use slab_blocks::get_slab_block_textures;
pub use stair_blocks::get_stair_block_textures;

mod normal_blocks;
mod rotate_4_way_blocks;
mod stair_blocks;
mod slab_blocks;

pub fn get_block_textures(texture_filtering_mode: &TextureFilteringMode, block_textures_path: &Utf8Path, filter: &[&str]) -> Vec<(String, TextureWithBlockState)> {
    filter.iter()
        .filter_map(|texture_info| {
            let (texture_name, block_id, block_state_properties) = match &texture_info.split("|").collect::<Vec<_>>()[..] {
                &[texture_name, block_id] => (texture_name, block_id, None),
                &[texture_name, block_id, block_state_properties] => (
                    texture_name,
                    block_id,
                    Some(block_state_properties.split(",")
                        .map(|property_definition| {
                            match &property_definition.split("=").collect::<Vec<_>>()[..] {
                                &[name, value] => (name.to_string(), value.to_string()),
                                _ => Err(eyre!("Invalid property definition '{}'.", property_definition)).unwrap()
                            }
                        })
                        .collect::<HashMap<_, _>>()
                    )
                ),
                _ => Err(eyre!("Invalid texture info line '{}'.", texture_info)).unwrap()
            };

            let texture_path = block_textures_path.join(format!("{texture_name}.png"));

            let result = match image::open(&texture_path) {
                Ok(texture) => {
                    let texture = texture.crop_imm(0, 0, 16, 16);
                    Some((texture_name.to_string(), texture))
                },
                Err(e) => {
                    tracing::warn!("Unable to find texture '{}': {e}", texture_path);
                    None
                }
            };

            match texture_filtering_mode {
                TextureFilteringMode::AllowList(allowed_textures) => {
                    result.and_then(|(name, texture)| {
                        if allowed_textures.contains(&name) {
                            Some((name, TextureWithBlockState {
                                texture,
                                block_id: block_id.into(),
                                block_state_properties,
                            }))
                        } else {
                            None
                        }
                    })
                }
                TextureFilteringMode::BlockList(blocked_textures) => {
                    result.and_then(|(name, texture)| {
                        if blocked_textures.contains(&name) {
                            None
                        } else {
                            Some((name, TextureWithBlockState {
                                texture,
                                block_id: block_id.into(),
                                block_state_properties,
                            }))
                        }
                    })
                }
            }
        })
        .collect::<Vec<_>>()
}

pub struct TextureWithBlockState {
    pub texture: DynamicImage,
    pub block_id: String,
    pub block_state_properties: Option<HashMap<String, String>>
}