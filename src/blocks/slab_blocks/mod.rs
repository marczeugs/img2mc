use std::collections::HashMap;

use camino::Utf8Path;
use color_eyre::eyre;
use image::Rgba;
use once_cell::sync::Lazy;

use crate::blocks;
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::TextureFilteringMode;
use crate::helpers::FillPixels;

pub static SLAB_BLOCKS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("blocks.txt").lines().collect::<Vec<_>>()
});

pub fn get_slab_block_textures(texture_filtering_mode: &TextureFilteringMode, block_textures_path: &Utf8Path) -> eyre::Result<HashMap<String, TextureWithBlockState>> {
    Ok(
        blocks::get_block_textures(texture_filtering_mode, block_textures_path, &SLAB_BLOCKS).into_iter()
            .flat_map(|(name, TextureWithBlockState { texture, block_id, .. })| {
                (0..2)
                    .map(|i| {
                        let mut texture = texture.clone();
                        texture.fill_pixels(0, i * 8, 16, 8, Rgba([0, 0, 0, 0]));

                        (format!("{}_slab_{}", &name, i * 180), TextureWithBlockState {
                            texture,
                            block_id: block_id.clone(),
                            block_state_properties: Some([("type".to_string(), if i == 0 { "bottom".to_string() } else { "top".to_string() })].into())
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<HashMap<_, _>>()
    )
}