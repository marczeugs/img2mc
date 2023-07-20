use std::str::FromStr;

use camino::Utf8PathBuf;

#[derive(gumdrop::Options)]
pub struct CliArguments {
    pub help: bool,

    #[options(help = "Path of an extracted <Minecraft JAR>/assets/minecraft/textures/block folder.", short = "t", meta = "<PATH>", required)]
    pub block_textures_path: Utf8PathBuf,

    #[options(help = "Image to be processed.", short = "i", meta = "<PATH/URL>", required)]
    pub input_image_path: String,

    #[options(help = "Path to desired output. For schematic output use .schematic or .litematic. Everything else is interpreted as image output.", short = "o", meta = "<PATH>", required)]
    pub output_path: Utf8PathBuf,

    #[options(help = "The width of the output Minecraft structure in blocks.", short = "w", meta = "<BLOCKS>")]
    pub block_width: Option<usize>,

    #[options(help = "The height of the output Minecraft structure in blocks.", short = "h", meta = "<BLOCKS>", default = "32")]
    pub block_height: usize,

    #[options(help = "The size of the grid each block texture gets split into for analysing. Higher values increase computation load.", short = "r", meta = "<NUMBER>", default = "4")]
    pub chunk_resolution: usize,

    #[options(help = "What dithering matrix to use. Options: JarvisJudiceNinke, FloydSteinberg", meta = "<ALGORITHM>", default = "JarvisJudiceNinke")]
    pub dithering_matrix: DitheringMatrix,

    #[options(help = "Exclude blocks that cannot be obtained in survival mode.", short = "s", default = "false")]
    pub exclude_non_survival_blocks: bool,

    #[options(help = "Limit the block palette to the provided textures. Takes precedent over exclude-non-survival-blocks.", short = "p")]
    pub block_palette: Option<BlockPalette>,
}

pub enum DitheringMatrix {
    JarvisJudiceNinke,
    FloydSteinberg,
}

impl DitheringMatrix {
    pub fn to_matrix(&self) -> Vec<Vec<usize>> {
        match self {
            DitheringMatrix::JarvisJudiceNinke => vec![
                vec![0, 0, 0, 7, 5],
                vec![3, 5, 7, 5, 3],
                vec![1, 3, 5, 3, 1],
            ],
            DitheringMatrix::FloydSteinberg => vec![
                vec![0, 0, 7],
                vec![3, 5, 1],
            ],
        }
    }
}

impl FromStr for DitheringMatrix {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JarvisJudiceNinke" => Ok(Self::JarvisJudiceNinke),
            "FloydSteinberg" => Ok(Self::FloydSteinberg),
            _ => Err("Invalid dithering algorithm.")
        }
    }
}

pub struct BlockPalette(pub Vec<String>);

impl FromStr for BlockPalette {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split(",").map(|s| s.into()).collect()))
    }
}

pub enum TextureFilteringMode {
    AllowList(Vec<String>),
    BlockList(Vec<String>),
}