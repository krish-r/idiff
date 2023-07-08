# idiff

diff - for images (compares images pixel by pixel)

## Note

-   I'm exploring the Rust language + libraries, so wanted to build something while learning.
-   So you might -

    -   see - `unwrap`'s, `expect`'s and instead of proper error handling in some places.
    -   not see - clean / optimized code or design patterns or full test coverage.

## Approach

-   Compare every pixel between the images for every block and check for differences, and display the comparison status.
-   if the `highlight` option is enabled, highlight the blocks with difference and store the output with the file name specified in the `output` option (default `TARGET_FILE_NAME_diff`).

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

### Option 1 - using Cargo

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

### Option 2 - using the binary from release page

```sh
# Download the binary from release page
curl -LO https://github.com/krish-r/idiff/releases/download/0.2.0/idiff_0.2.0.tar.gz

# Extract the archive
tar xvzf ./idiff_0.2.0.tar.gz && rm -ir ./idiff_0.2.0.tar.gz

# add executable permission to user
chmod u+x ./idiff

# Move the file somewhere in your `$PATH` (for ex. `~/.local/bin`)
mv ./idiff ~/.local/bin/idiff
```

## Uninstall instructions

```sh
# If the repository was cloned
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
