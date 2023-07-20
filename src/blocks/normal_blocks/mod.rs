use std::collections::HashMap;

use camino::Utf8Path;
use color_eyre::eyre;
use once_cell::sync::Lazy;

use crate::blocks;
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::TextureFilteringMode;

pub static NORMAL_BLOCK_NAMES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("blocks.txt").lines().collect::<Vec<_>>()
});

pub fn get_normal_block_textures(texture_filtering_mode: &TextureFilteringMode, block_textures_path: &Utf8Path) -> eyre::Result<HashMap<String, TextureWithBlockState>> {
    Ok(
        blocks::get_block_textures(texture_filtering_mode, block_textures_path, &NORMAL_BLOCK_NAMES).into_iter()
            .collect::<HashMap<_, _>>()
    )
}