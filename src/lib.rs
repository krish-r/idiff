use std::path::{Path, PathBuf};

use clap::Parser;
use colored::*;
use image::GenericImage;

/// Represents the (width, height) tuple.
type Dimensions = (u32, u32);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// source file name
    #[arg(long, value_name = "SOURCE_FILE_NAME")]
    src: PathBuf,

    /// target file name
    #[arg(long, value_name = "TARGET_FILE_NAME")]
    tgt: PathBuf,

    /// strict comparison (exits if dimensions are different)
    #[arg(long)]
    strict: bool,

    /// highlight differences in a new file
    #[arg(long)]
    highlight: bool,

    /// pixel block size for highlighting difference
    #[arg(long, requires = "highlight", default_value_t = 10)]
    block: u32,

    /// optional output file name (without extension)
    #[arg(short, long, value_name = "OUTPUT_FILE_NAME", requires = "highlight")]
    output: Option<String>,
}

pub fn run() {
    let cli = Cli::parse();

    if !cli.src.exists() || !cli.tgt.exists() {
        eprintln!(
            "{}",
            "Invalid values for src/tgt path. Please check and try again.".red()
        );
        std::process::exit(1);
    }

    let (src, tgt) = match (image::open(&cli.src), image::open(&cli.tgt)) {
        (Ok(s), Ok(t)) => (s.to_rgba8(), t.to_rgba8()),
        (_, _) => {
            eprintln!(
                "{}",
                "Encountered error while opening source / target image.".red()
            );
            std::process::exit(1);
        }
    };

    let src_dimension: Dimensions = src.dimensions();
    let tgt_dimension: Dimensions = tgt.dimensions();

    if cli.strict && !same_dimensions(&src_dimension, &tgt_dimension) {
        eprintln!("{}",
            format!("'src' ({:?}) & 'tgt' ({:?}) do not have the same dimensions. (Try without 'strict' flag to check the differences)", src_dimension, tgt_dimension)
            .red());
        std::process::exit(1);
    }

    let bounds = match Bounds::get_max_bounds(src_dimension, tgt_dimension) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let diff = percentage_difference(&src, &tgt, &bounds);

    if diff == 0.0 {
        println!(
            "{}",
            "Comparison Completed. No difference observed between the images!".green()
        );
        std::process::exit(0);
    } else {
        println!(
            "A difference of '{:.5}{}' is observed between images.",
            diff.to_string().red(),
            "%".red()
        );
        if !cli.highlight {
            println!("{}", "(Difference highlighting is currently disabled. Try with 'highlight' flag to highlight the differences)".yellow());
            std::process::exit(0);
        }
    }

    let mut tgt_copy = match copy_image(&tgt) {
        Ok(t) => t,
        Err(_) => {
            eprintln!(
                "{}",
                "Encountered error while creating a copy of target image for highlighting.".red()
            );
            std::process::exit(1);
        }
    };

    highlight_difference(&src, &mut tgt_copy, &bounds, cli.block);

    let output = generate_output_file_name(cli.output, &cli.tgt).unwrap();
    tgt_copy.save(&output).unwrap();
    println!(
        "{}",
        format!("Output written into {}", &output.to_str().unwrap()).green()
    );
}

/// Checks if the given dimensions are same.
fn same_dimensions(src: &Dimensions, tgt: &Dimensions) -> bool {
    matches!(src.cmp(tgt), std::cmp::Ordering::Equal)
}

/// Creates a copy of the image
fn copy_image(img: &image::RgbaImage) -> Result<image::RgbaImage, image::error::ImageError> {
    let mut img_copy: image::RgbaImage =
        image::ImageBuffer::new(img.dimensions().0, img.dimensions().1);
    img_copy.copy_from(img, 0, 0)?;
    Ok(img_copy)
}

/// Compare the pixel difference for the specified bounds between the images and calculate the percentage difference.
///
/// Logic: `(mismatching pixels / total pixels ) * 100`
fn percentage_difference(src: &image::RgbaImage, tgt: &image::RgbaImage, bounds: &Bounds) -> f64 {
    let mut pixel_difference = 0;

    for y in bounds.min_height..bounds.max_height {
        for x in bounds.min_width..bounds.max_width {
            if src.get_pixel(x, y) != tgt.get_pixel(x, y) {
                pixel_difference += 1;
            }
        }
    }
    let total_pixels = bounds.max_height * bounds.max_width;
    (pixel_difference as f64 / total_pixels as f64) * 100.0
}

/// Compare the pixel difference for every pixel block for the specified bounds between the images and calculate the percentage difference.
fn highlight_difference(
    src: &image::RgbaImage,
    tgt: &mut image::RgbaImage,
    bounds: &Bounds,
    block: u32,
) {
    for start_height in (0..bounds.max_height).step_by(block as usize) {
        for start_width in (0..bounds.max_width).step_by(block as usize) {
            // Note: max width & height should not exceed the overall bounds
            let max_width = std::cmp::min(start_width + block, bounds.max_width);
            let max_height = std::cmp::min(start_height + block, bounds.max_height);

            let current_dimension = Bounds::new(start_width, max_width, start_height, max_height);
            let diff = percentage_difference(src, tgt, &current_dimension);
            if diff != 0.0 {
                for x in start_width..max_width {
                    let pixel = tgt.get_pixel_mut(x, start_height);
                    *pixel = image::Rgba([255, 0, 0, 255]);

                    let pixel = tgt.get_pixel_mut(x, max_height - 1);
                    *pixel = image::Rgba([255, 0, 0, 255]);
                }

                for y in start_height..max_height {
                    let pixel = tgt.get_pixel_mut(start_width, y);
                    *pixel = image::Rgba([255, 0, 0, 255]);

                    let pixel = tgt.get_pixel_mut(max_width - 1, y);
                    *pixel = image::Rgba([255, 0, 0, 255]);
                }
            }
        }
    }
}

/// Generate output file name with extension if one is provided else use the backup file.
fn generate_output_file_name(output: Option<String>, backup_file: &Path) -> Option<PathBuf> {
    let file_name = match output {
        Some(f) => f,
        None => format!("{}_diff", backup_file.file_stem()?.to_str()?.to_owned()),
    };

    let mut output = backup_file.with_file_name(file_name);
    if let Some(ext) = backup_file.extension() {
        output.set_extension(ext);
    }

    Some(output)
}

#[derive(Debug, PartialEq)]
struct Bounds {
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

impl Bounds {
    fn new(min_width: u32, max_width: u32, min_height: u32, max_height: u32) -> Bounds {
        Bounds {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }
    /// Get the max bounds from the provided width
    fn get_max_bounds(src: Dimensions, tgt: Dimensions) -> Result<Bounds, String> {
        let (w1, h1) = src;
        let (w2, h2) = tgt;

        let max_width = std::cmp::min(w1, w2);
        let max_height = std::cmp::min(h1, h2);

        if max_width == 0 || max_height == 0 {
            return Err(String::from("Maximum width / height cannot be ZERO (0)."));
        }

        Ok(Bounds {
            min_width: 0,
            max_width,
            min_height: 0,
            max_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_false_for_mismatching_dimensions() {
        let src = (0, 0);
        let tgt = (1, 1);

        assert!(!same_dimensions(&src, &tgt));
    }

    #[test]
    fn should_return_true_for_mismatching_dimensions() {
        let src = (1, 1);
        let tgt = (1, 1);

        assert!(same_dimensions(&src, &tgt));
    }

    #[test]
    fn should_return_zero_pct_diff_for_matching_images() {
        let src = image::ImageBuffer::new(100, 100);
        let tgt = image::ImageBuffer::new(100, 100);
        let bounds = Bounds::new(0, 100, 0, 100);

        assert_eq!(0.0, percentage_difference(&src, &tgt, &bounds));
    }

    #[test]
    fn should_return_non_zero_pct_diff_for_images_with_differences() {
        let src = image::ImageBuffer::new(100, 100);

        let mut tgt = image::ImageBuffer::new(100, 100);
        *tgt.get_pixel_mut(10, 10) = image::Rgba([10, 10, 10, 255]);
        *tgt.get_pixel_mut(20, 20) = image::Rgba([10, 10, 10, 255]);

        let bounds = Bounds::new(0, 100, 0, 100);

        assert_eq!(0.02, percentage_difference(&src, &tgt, &bounds));
    }

    #[test]
    fn should_return_err_for_zero_bounds() {
        let src = (0, 0);
        let tgt = (1, 1);

        assert_eq!(
            Err(String::from("Maximum width / height cannot be ZERO (0).")),
            Bounds::get_max_bounds(src, tgt)
        );
    }

    #[test]
    fn should_return_ok_for_non_zero_bounds() {
        let src = (10, 100);
        let tgt = (100, 10);

        assert_eq!(
            Ok(Bounds::new(0, 10, 0, 10)),
            Bounds::get_max_bounds(src, tgt)
        );
    }

    #[test]
    fn should_generate_backup_file_name_by_default() {
        assert_eq!(
            Some(PathBuf::from("/target_test_diff.png")),
            generate_output_file_name(None, &PathBuf::from("/target_test.png"))
        );
    }

    #[test]
    fn should_generate_output_file_name_when_option_is_provided() {
        assert_eq!(
            Some(PathBuf::from("/custom_output_file.png")),
            generate_output_file_name(
                Some(String::from("custom_output_file")),
                &PathBuf::from("/target_test.png"),
            )
        );
    }

    #[test]
    pub fn should_highlight_when_differences_are_observed() {
        let src = image::ImageBuffer::new(100, 100);

        let mut tgt = image::ImageBuffer::new(100, 100);
        *tgt.get_pixel_mut(15, 15) = image::Rgba([10, 10, 10, 255]);
        *tgt.get_pixel_mut(55, 55) = image::Rgba([10, 10, 10, 255]);

        let mut tgt_clone1 = tgt.clone();
        let mut tgt_clone2 = tgt.clone();

        let bounds = Bounds::new(0, 20, 0, 20);

        highlight_difference(&src, &mut tgt_clone1, &bounds, 10);

        for i in 10..20 {
            *tgt_clone2.get_pixel_mut(i, 10) = image::Rgba([255, 0, 0, 255]);
            *tgt_clone2.get_pixel_mut(i, 19) = image::Rgba([255, 0, 0, 255]);
            *tgt_clone2.get_pixel_mut(10, i) = image::Rgba([255, 0, 0, 255]);
            *tgt_clone2.get_pixel_mut(19, i) = image::Rgba([255, 0, 0, 255]);
        }

        assert_ne!(tgt, tgt_clone1);
        assert_eq!(tgt_clone2, tgt_clone1);
    }
}
