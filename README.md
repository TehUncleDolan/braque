# Braque - Scramble/Unscramble Images

[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Overview

Split image into tiles and scramble/unscramble them based on a seed.

It can be used via a command-line interface or as a library in other Rust
programs.

## Installing

Pre-compiled binaries can be downloaded from the
[Releases](https://github.com/TehUncleDolan/braque/releases/) page.

Alternatively, braque can be installed from Cargo, via the following command:

```
cargo install braque --features cli
```

Braque can be built from source using the latest stable or nightly Rust.
This is primarily useful for developing on braque.

```
git clone https://github.com/TehUncleDolan/braque.git
cd braque
cargo build --release --features cli
cp target/release/braque /usr/local/bin
```

Braque follows Semantic Versioning.

## Library Usage

```rust
let block_size = BlockSize::try_from(50).expect("valid block size");
let seed = "SECRET";

let output = scramble(&input_image, block_size, seed.as_bytes());
let original = unscramble(&output, block_size, seed.as_bytes());
```

## Usage

Braque can also be used as a command-line utility. Basic usage looks similar to the
following:

```
braque --mode scramble --seed SECRET -b 50 -i foo.png -o foo-scrambled.png
```

`seed` is used to randomized the scrambling (the same seed must be used to
restore the original image).

More details can be found by running `braque -h`.

## Credits

* Braque is a Rust version of [Pycasso](https://github.com/catsital/pycasso),
itself inspired by
[image-scramble](https://github.com/webcaetano/image-scramble).

* Sample image is taken from [Pepper&Carrot](https://peppercarrot.com/) by David Revoy licensed under [CC BY 4.0](https://www.peppercarrot.com/en/license/index.html).
