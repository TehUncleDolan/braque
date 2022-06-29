use braque::BlockSize;
use image::{io::Reader as ImageReader, DynamicImage, ImageOutputFormat};
use std::io::Cursor;

const SEED: &[u8] = b"Braque";

#[test]
fn scramble() {
    let block_size = BlockSize::try_from(33).expect("valid size");
    let (input, expected) = load_test_and_ref("ORIGINAL", "SCRAMBLED");

    let result = braque::scramble(&input, block_size, SEED);

    assert_eq!(image_to_bytes(&result), expected);
}

#[test]
fn unscramble() {
    let block_size = BlockSize::try_from(33).expect("valid size");
    let (input, expected) = load_test_and_ref("SCRAMBLED", "UNSCRAMBLED");

    let result = braque::unscramble(&input, block_size, SEED);

    assert_eq!(image_to_bytes(&result), expected);
}

#[test]
fn roundtrip() {
    let block_size = BlockSize::try_from(33).expect("valid size");
    let (input, expected) = load_test_and_ref("ORIGINAL", "ORIGINAL");

    let scrambled = braque::scramble(&input, block_size, SEED);
    let result = braque::unscramble(&scrambled, block_size, SEED);

    assert_eq!(image_to_bytes(&result), expected);
}

// Returns test image and expected bytes.
fn load_test_and_ref(input: &str, reference: &str) -> (DynamicImage, Vec<u8>) {
    let datadir = format!("{}/testdata", env!("CARGO_MANIFEST_DIR"));
    let input_path = format!("{datadir}/Pepper-and-Carrot_by-David-Revoy_E05P01_p2-{input}.png",);
    let input = ImageReader::open(&input_path)
        .expect("cannot open input image")
        .decode()
        .expect("cannot decode input image");

    let reference_path =
        format!("{datadir}/Pepper-and-Carrot_by-David-Revoy_E05P01_p2-{reference}.png",);
    let expected = std::fs::read(&reference_path).expect("read ref file");

    (input, expected)
}

fn image_to_bytes(img: &DynamicImage) -> Vec<u8> {
    let mut buff = Cursor::new(Vec::new());
    img.write_to(&mut buff, ImageOutputFormat::Png)
        .expect("write image in memory");

    buff.into_inner()
}
