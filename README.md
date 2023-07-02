# idiff

diff - for images (compares images pixel by pixel)

## Note

-   I'm exploring the Rust language + libraries, so wanted to build something while learning.
-   So you might -

    -   see - `unwrap`'s, `expect`'s and instead of proper error handling in some places.
    -   not see - clean / optimized code or design patterns or full test coverage.

## Approach

This takes a naive two pass approach -

-   first pass - compare every pixel between the images and check for differences, and display the comparison status.
-   second pass - (if the `highlight` option is enabled)
    -   compare every pixel in each block (default: `10x10` pixels) and check for differences, and if there is any, highlight the block.
    -   store the output with the file name specified in the `output` option (default `TARGET_FILE_NAME_diff`).

## Dependencies

-   clap
-   colored
-   image

### Testing Dependencies

-   assert_cmd
-   assert_fs
-   insta
-   predicates

## Installation instructions

```sh
# Clone the repository
git clone https://github.com/krish-r/idiff.git

# Switch to the cloned directory
cd idiff/

# Try it without installing
cargo run -- [OPTIONS]
# For Ex.
#   cargo run -- --help
#   cargo run -- --src <SOURCE_FILE_NAME> --tgt <TARGET_FILE_NAME>

# or

# To install (**Note**: the cargo bin directory `~/.cargo/bin` should be in your `$PATH`)
cargo install --path .
```

## Uninstall instructions

```sh
rm -r idiff/

# If the binary was installed
rm -i $(which idiff)
```

## Usage

```sh
idiff --help
```

```sh
Usage: idiff [OPTIONS] --src <SOURCE_FILE_NAME> --tgt <TARGET_FILE_NAME>

Options:
      --src <SOURCE_FILE_NAME>     source file name
      --tgt <TARGET_FILE_NAME>     target file name
      --strict                     strict comparison (exits if dimensions are different)
      --highlight                  highlight differences in a new file
      --block <BLOCK>              pixel block size for highlighting difference [default: 10]
  -o, --output <OUTPUT_FILE_NAME>  optional output file name (without extension)
  -h, --help                       Print help
  -V, --version                    Print version
```
