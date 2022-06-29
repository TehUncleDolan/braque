use braque::{scramble, BlockSize};
use image::io::Reader as ImageReader;

fn main() {
    let input = ImageReader::open("foo.png")
        .expect("cannot open input image")
        .decode()
        .expect("cannot decode input image");
    let block_size = BlockSize::try_from(50).expect("valid size");
    let seed = "SECRET";

    let output = scramble(&input, block_size, seed.as_bytes());

    output
        .save("foo-scrambled.png")
        .expect("cannot write output image");
}
