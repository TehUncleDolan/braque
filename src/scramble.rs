use crate::shuffle::{shuffle, unshuffle};
use image::{math::Rect, DynamicImage};
use std::cmp;

/// An image block size (must be strictly positive).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BlockSize(u32);

impl TryFrom<u32> for BlockSize {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err("BlockSize only accepts value greater than zero");
        }
        Ok(Self(value))
    }
}

impl From<BlockSize> for u32 {
    fn from(value: BlockSize) -> Self {
        value.0
    }
}

/// Restores an image splitted in `block_size` blocks and scrambled with `seed`.
#[must_use]
pub fn scramble(img: &DynamicImage, block_size: BlockSize, seed: &[u8]) -> DynamicImage {
    rearrange(img, block_size, seed, Mode::Scramble)
}

/// Splits an image into `block_size` blocks and scrambles it using `seed`.
#[must_use]
pub fn unscramble(img: &DynamicImage, block_size: BlockSize, seed: &[u8]) -> DynamicImage {
    rearrange(img, block_size, seed, Mode::Unscramble)
}

/// Operation mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    /// Scramble the image.
    Scramble,
    /// Unscramble the image.
    Unscramble,
}

/// Rearrages the input image according to the specified mode.
fn rearrange(img: &DynamicImage, block_size: BlockSize, seed: &[u8], mode: Mode) -> DynamicImage {
    let mut canvas = img.clone();
    let regions = Regions::new(img.width(), img.height(), block_size);

    // Rearrage each region.
    for region in [regions.main, regions.right, regions.bottom]
        .into_iter()
        .flatten()
    {
        for (src, dst) in region.rearrange(seed, mode) {
            copy_paste(img, &mut canvas, src, dst);
        }
    }

    canvas
}

/// Copy `src` block from `src_img` onto `dst` block in `dst_img`.
fn copy_paste(src_img: &DynamicImage, dst_img: &mut DynamicImage, src: Rect, dst: Rect) {
    let block = src_img.crop_imm(src.x, src.y, src.width, src.height);
    image::imageops::overlay(dst_img, &block, dst.x.into(), dst.y.into());
}

/// Image regions.
///
/// The image is divided into four regions at most:
/// - a region of square chunk of `block_size`, covers most if not all (when
///   image size is a multiple of the block size) of the image.
/// - a column of smaller width blocks on the right of the image (happens when
///   the image width is not a multiple of the block size).
/// - a row of smaller height blocks at the bottom of the image (happens when
///   the image height is not a multiple of the block size).
/// - a single smaller block on the bottom right corner (happens when both image
///   width and height are not multiple of the block size).
///
/// The fourth one is ignored as it can only map to itself in the destination.
#[derive(Debug)]
struct Regions {
    /// Main region, of square blocks.
    main: Option<Region>,
    /// Right edge, of blocks of smaller width.
    right: Option<Region>,
    /// Bottom edge, of blocks of smaller height.
    bottom: Option<Region>,
}

impl Regions {
    /// Computes regions for an image split in chunk of `block_size`.
    fn new(img_width: u32, img_height: u32, block_size: BlockSize) -> Self {
        let block_size = u32::from(block_size);
        // XXX: Waiting for https://github.com/rust-lang/rust/issues/88581.
        let nb_rows = num_integer::Integer::div_ceil(&img_height, &block_size);
        let nb_cols = num_integer::Integer::div_ceil(&img_width, &block_size);
        let (main, right, bottom) = get_blocks(img_width, img_height, nb_rows, nb_cols, block_size);

        // Right region is either missing (0 cols) or the rightmost column (1).
        let right_cols = u32::from(!right.is_empty());
        // Main covers the whole width, except when there is a right region.
        let main_cols = nb_cols - right_cols;
        // Ditto for bottom region.
        let bottom_cols = main_cols;

        // Build regions.
        Self {
            main: (!main.is_empty()).then(|| Region::new(main, main_cols)),
            right: (!right.is_empty()).then(|| Region::new(right, right_cols)),
            bottom: (!bottom.is_empty()).then(|| Region::new(bottom, bottom_cols)),
        }
    }
}

/// Computes image blocks and group them by size.
fn get_blocks(
    img_width: u32,
    img_height: u32,
    nb_rows: u32,
    nb_cols: u32,
    block_size: u32,
) -> (Vec<Rect>, Vec<Rect>, Vec<Rect>) {
    let mut main = Vec::new();
    let mut right = Vec::new();
    let mut bottom = Vec::new();

    for i in 0..nb_rows * nb_cols {
        let row = i / nb_cols;
        let col = i % nb_cols;
        let x = col * block_size;
        let y = row * block_size;

        let block = Rect {
            x,
            y,
            width: cmp::min(img_width - x, block_size),
            height: cmp::min(img_height - y, block_size),
        };

        match (block.width == block_size, block.height == block_size) {
            // Square block => main region.
            (true, true) => {
                main.push(block);
            }
            // Small-width block => right edge.
            (false, true) => {
                assert_eq!(col, nb_cols - 1, "misplaced small-width block");
                right.push(block);
            }
            // Small-height block => bottom edge.
            (true, false) => {
                assert_eq!(row, nb_rows - 1, "misplaced small-height block");
                bottom.push(block);
            }
            // Smaller block => the bottom-right corner.
            (false, false) => {
                assert!(
                    col == nb_cols - 1 && row == nb_rows - 1,
                    "misplaced small block: {col}/{nb_cols} and {row}/{nb_rows}"
                );
                // No need to keep track of the corner since it doesn't move.
            }
        }
    }

    (main, right, bottom)
}

/// An image region composed of homogeneous blocks.
#[derive(Debug)]
struct Region {
    /// Image blocks.
    blocks: Vec<Rect>,
    /// Number of columns (in blocks).
    nb_cols: u32,
}

impl Region {
    /// Initialize a new region.
    fn new(blocks: Vec<Rect>, nb_cols: u32) -> Self {
        assert!(!blocks.is_empty(), "region cannot be empty");
        assert!(nb_cols > 0, "region must have a width");

        Self { blocks, nb_cols }
    }

    /// Computes the rearrangement of the block according to the given mode.
    ///
    /// Returns a stream of rectangle pairs (source, destination) that can be
    /// used to build the output image by copy/pasting blocks accordingly.
    #[allow(clippy::cast_possible_truncation)] // Indices fits in u32 here.
    fn rearrange(&self, seed: &[u8], mode: Mode) -> impl Iterator<Item = (Rect, Rect)> + '_ {
        // Identify the top-right corner of the region, used as origin.
        let start_x = self.blocks[0].x;
        let start_y = self.blocks[0].y;
        // Shuffle the blocks indices to compute the desired transformation.
        let indices = (0..self.blocks.len()).map(|i| i as u32).collect::<Vec<_>>();
        let shuffled_indices = match mode {
            Mode::Scramble => shuffle(&indices, seed),
            Mode::Unscramble => unshuffle(&indices, seed),
        };

        // Find each block's source using the shuffled list
        self.blocks.iter().enumerate().map(move |(i, block)| {
            let j = shuffled_indices[i];
            let row = j / self.nb_cols;
            let col = j % self.nb_cols;
            let x = col * block.width;
            let y = row * block.height;

            let src = Rect {
                x: start_x + x,
                y: start_y + y,
                width: block.width,
                height: block.height,
            };

            (src, *block)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_size() {
        assert_eq!(BlockSize::try_from(50), Ok(BlockSize(50)));
        assert!(BlockSize::try_from(0).is_err());
    }

    // Image resolution is a multiple of the block size.
    #[test]
    fn image_match_block() {
        let regions = Regions::new(800, 600, BlockSize(10));

        assert_eq!(regions.main.expect("main region").blocks.len(), 80 * 60);
        assert!(regions.right.is_none());
        assert!(regions.bottom.is_none());
    }

    // Image width isn't a multiple of the block size.
    #[test]
    fn image_width_mismatch() {
        let regions = Regions::new(800, 600, BlockSize(30));

        assert_eq!(regions.main.expect("main region").blocks.len(), 26 * 20);
        assert_eq!(regions.right.expect("right region").blocks.len(), 20);
        assert!(regions.bottom.is_none());
    }

    // Image height isn't a multiple of the block size.
    #[test]
    fn image_height_mismatch() {
        let regions = Regions::new(800, 600, BlockSize(80));

        assert_eq!(regions.main.expect("main region").blocks.len(), 10 * 7);
        assert!(regions.right.is_none());
        assert_eq!(regions.bottom.expect("bottom region").blocks.len(), 10);
    }

    // Image resolution isn't a multiple of the block size.
    #[test]
    fn image_mismatch_block() {
        let regions = Regions::new(800, 600, BlockSize(70));

        assert_eq!(regions.main.expect("main region").blocks.len(), 11 * 8);
        assert_eq!(regions.right.expect("right region").blocks.len(), 8);
        assert_eq!(regions.bottom.expect("bottom region").blocks.len(), 11);
    }

    // Image width smaller than block size.
    #[test]
    fn image_width_too_small() {
        let regions = Regions::new(80, 600, BlockSize(100));

        assert!(regions.main.is_none());
        assert_eq!(regions.right.expect("right region").blocks.len(), 6);
        assert!(regions.bottom.is_none());
    }

    // Image height smaller than block size.
    #[test]
    fn image_height_too_small() {
        let regions = Regions::new(800, 60, BlockSize(100));

        assert!(regions.main.is_none());
        assert!(regions.right.is_none());
        assert_eq!(regions.bottom.expect("bottom region").blocks.len(), 8);
    }

    // Image smaller than block size.
    #[test]
    fn image_too_small() {
        let regions = Regions::new(80, 60, BlockSize(100));

        // We have a single block: no transformation and thus no region.
        assert!(regions.main.is_none());
        assert!(regions.right.is_none());
        assert!(regions.bottom.is_none());
    }
}
