use braque::{scramble, unscramble, BlockSize};
use clap::{ArgEnum, Parser};
use eyre::{eyre, WrapErr};
use image::io::Reader as ImageReader;
use std::path::PathBuf;

// Operation mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq, ArgEnum)]
enum Mode {
    // Scramble the image.
    Scramble,
    // Unscramble the image.
    Unscramble,
}

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Path to the input image.
    #[clap(short, long)]
    input: PathBuf,
    /// Path to the output image.
    #[clap(short, long)]
    output: PathBuf,
    /// Scrambling mode.
    #[clap(short, long, value_parser)]
    mode: Mode,
    /// Size (in pixels) of block to chunk an image
    #[clap(short, long, default_value_t = 50)]
    block_size: u32,
    /// Seed to use to (un)scramble an image.
    #[clap(short, long, default_value_t=String::from("braque"))]
    seed: String,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let img = ImageReader::open(&args.input)
        .with_context(|| format!("open {}", args.input.display()))?
        .decode()
        .with_context(|| format!("decode {}", args.input.display()))?;

    let block_size =
        BlockSize::try_from(args.block_size).map_err(|err| eyre!("invalid block size: {err}"))?;
    let result = match args.mode {
        Mode::Scramble => scramble(&img, block_size, args.seed.as_bytes()),
        Mode::Unscramble => unscramble(&img, block_size, args.seed.as_bytes()),
    };

    result
        .save(&args.output)
        .with_context(|| format!("write {}", args.output.display()))?;

    Ok(())
}
