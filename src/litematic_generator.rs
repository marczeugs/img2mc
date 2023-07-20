use std::collections::HashMap;
use std::time;
use std::time::SystemTime;
use color_eyre::eyre;
use fastnbt::LongArray;
use itertools::Itertools;
use serde::Serialize;
use crate::blocks::TextureWithBlockState;
use crate::cli_arguments::CliArguments;

pub fn make_bytes(block_width: usize, cli_arguments: &CliArguments, output_blocks: &Vec<Vec<String>>, block_textures_and_states: &HashMap<String, TextureWithBlockState>) -> eyre::Result<Vec<u8>> {
    let block_count = block_width * cli_arguments.block_height;

    let air_block_list = vec!["air".to_string()];

    let used_block_textures = air_block_list.iter()
        .chain(output_blocks.iter().flat_map(|column| column))
        .unique()
        .collect::<Vec<_>>();

    let block_state_palette_by_texture = used_block_textures.iter()
        .enumerate()
        .map(|(index, &texture_name)| (texture_name, index))
        .collect::<HashMap<_, _>>();

    let block_states_info = used_block_textures.iter()
        .map(|&block_texture_name| (
            block_textures_and_states[block_texture_name].block_id.clone(),
            block_textures_and_states[block_texture_name].block_state_properties.clone()
        ))
        .collect::<Vec<_>>();

    Ok(fastnbt::to_bytes(&Schematic {
        minecraft_data_version: 3465,
        sub_version: 1,
        version: 6,
        metadata: Metadata {
            enclosing_size: XYZ {
                x: block_width as i32,
                y: cli_arguments.block_height as i32,
                z: 1,
            },
            region_count: 1,
            total_blocks: (block_count - output_blocks.iter().flat_map(|column| column).filter(|&block| block == "air").count()) as i32,
            total_volume: block_count as i32,
            time_created: SystemTime::now().duration_since(time::UNIX_EPOCH)?.as_millis() as i64,
            time_modified: SystemTime::now().duration_since(time::UNIX_EPOCH)?.as_millis() as i64,
            author: "img2mc".into(),
            description: "Generated by img2mc".into(),
            name: cli_arguments.output_path.file_stem().unwrap_or("image").into(),
        },
        regions: HashMap::from([
            ("Unnamed".into(), Region {
                position: XYZ {
                    x: 0,
                    y: 0,
                    z: 0,
                },
                size: XYZ {
                    x: block_width as i32,
                    y: cli_arguments.block_height as i32,
                    z: 1,
                },
                block_state_palette: block_states_info.into_iter()
                    .map(|(block_id, block_state_properties)| {
                        BlockStatePaletteEntry {
                            name: block_id,
                            properties: block_state_properties,
                        }
                    })
                    .collect(),
                entities: vec![],
                pending_block_ticks: vec![],
                pending_fluid_ticks: vec![],
                tile_entities: vec![],
                block_states: {
                    let bits_per_block = ((used_block_textures.len() as f32).log2().ceil() as usize).max(2);

                    let mut longs = vec![0i64; bits_per_block * block_width * cli_arguments.block_height / 64 + 1];

                    let mut bit_index = 0;

                    for y in (0..cli_arguments.block_height).rev() {
                        for x in 0..block_width {
                            longs[bit_index / 64] |= (block_state_palette_by_texture[&output_blocks[x][y]] as i64) << bit_index % 64;

                            if bit_index % 64 + bits_per_block >= 64 {
                                let written_bits = 64 - bit_index % 64;

                                bit_index += written_bits;
                                longs[bit_index / 64] |= (block_state_palette_by_texture[&output_blocks[x][y]] as i64) >> written_bits;
                                bit_index += bits_per_block - written_bits;
                            } else {
                                bit_index += bits_per_block;
                            }
                        }
                    }

                    LongArray::new(longs)
                }
            })
        ]),
    })?)
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Schematic {
    minecraft_data_version: i32,
    sub_version: i32,
    version: i32,
    metadata: Metadata,
    regions: HashMap<String, Region>
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Metadata {
    enclosing_size: XYZ,
    region_count: i32,
    total_blocks: i32,
    total_volume: i32,
    time_created: i64,
    time_modified: i64,
    author: String,
    description: String,
    name: String
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Region {
    position: XYZ,
    size: XYZ,
    block_state_palette: Vec<BlockStatePaletteEntry>,
    entities: Vec<EmptyObject>,
    pending_block_ticks: Vec<EmptyObject>,
    pending_fluid_ticks: Vec<EmptyObject>,
    tile_entities: Vec<EmptyObject>,
    block_states: LongArray,
}

#[derive(Serialize)]
struct XYZ {
    x: i32,
    y: i32,
    z: i32
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct BlockStatePaletteEntry {
    name: String,
    properties: Option<HashMap<String, String>>
}

#[derive(Serialize)]
struct EmptyObject;