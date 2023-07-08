use std::path::{Path, PathBuf};

use clap::Parser;
use colored::*;
use image::GenericImage;

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

    let src_dimension: Dimensions = Dimensions::from(src.dimensions());
    let tgt_dimension: Dimensions = Dimensions::from(tgt.dimensions());

    if cli.strict && !Dimensions::same(&src_dimension, &tgt_dimension) {
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

    if !bounds.is_greater_than(cli.block * cli.block) {
        eprintln!(
            "{}",
            format!(
                "block size ({:?}) cannot be greater than the max bound (height: {:?},  width: {:?}).",
                cli.block, bounds.max_height, bounds.max_width
            )
            .red()
        );
        std::process::exit(1);
    }

    let (diff, bounds_with_diff) = percentage_difference(&src, &tgt, &bounds, cli.block);

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

    highlight(&mut tgt_copy, bounds_with_diff);

    let output = generate_output_file_name(cli.output, &cli.tgt).unwrap();
    tgt_copy.save(&output).unwrap();
    println!(
        "{}",
        format!("Output written into {}", &output.to_str().unwrap()).green()
    );
}

/// Creates a copy of the image.
fn copy_image(img: &image::RgbaImage) -> Result<image::RgbaImage, image::error::ImageError> {
    let mut img_copy: image::RgbaImage =
        image::ImageBuffer::new(img.dimensions().0, img.dimensions().1);
    img_copy.copy_from(img, 0, 0)?;
    Ok(img_copy)
}

/// Compare the pixel difference for every pixel for the specified bounds between the images and calculate the percentage difference.
///
/// Returns the percentage difference and Vec\<Bounds\> where the difference was observed.
///
/// Logic: `(mismatching pixels / total pixels ) * 100`
fn percentage_difference(
    src: &image::RgbaImage,
    tgt: &image::RgbaImage,
    bounds: &Bounds,
    block: u32,
) -> (f32, Vec<Bounds>) {
    let mut total_diff = 0;
    let mut bounds_with_difference = Vec::new();

    for start_height in (bounds.min_height..bounds.max_height).step_by(block as usize) {
        for start_width in (bounds.min_width..bounds.max_width).step_by(block as usize) {
            // Note: max width & height should not exceed the overall bounds
            let max_width = std::cmp::min(start_width + block, bounds.max_width);
            let max_height = std::cmp::min(start_height + block, bounds.max_height);

            let current_bound = Bounds::new(start_width, max_width, start_height, max_height);
            let diff = pixel_difference(src, tgt, &current_bound);
            if diff != 0 {
                total_diff += diff;
                bounds_with_difference.push(current_bound);
            }
        }
    }
    let diff_percentage =
        ((total_diff as f32) / ((bounds.max_height * bounds.max_width) as f32)) * 100.0;
    (diff_percentage, bounds_with_difference)
}

/// Compare the pixel difference for the specified bounds between the images.
fn pixel_difference(src: &image::RgbaImage, tgt: &image::RgbaImage, bounds: &Bounds) -> u32 {
    let mut diff = 0;

    for y in bounds.min_height..bounds.max_height {
        for x in bounds.min_width..bounds.max_width {
            if src.get_pixel(x, y) != tgt.get_pixel(x, y) {
                diff += 1;
            }
        }
    }

    diff
}

/// Highlight the specified bounds in the image.
fn highlight(img: &mut image::RgbaImage, bounds: Vec<Bounds>) {
    for bound in bounds {
        for x in bound.min_width..bound.max_width {
            *img.get_pixel_mut(x, bound.min_height) = image::Rgba([255, 0, 0, 255]);
            *img.get_pixel_mut(x, bound.max_height - 1) = image::Rgba([255, 0, 0, 255]);
        }

        for y in bound.min_height..bound.max_height {
            *img.get_pixel_mut(bound.min_width, y) = image::Rgba([255, 0, 0, 255]);
            *img.get_pixel_mut(bound.max_width - 1, y) = image::Rgba([255, 0, 0, 255]);
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

/// Represents the Dimension (width, height).
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Dimensions(u32, u32);

impl Dimensions {
    /// Create Dimensions from a tuple.
    fn from(d: (u32, u32)) -> Dimensions {
        Dimensions(d.0, d.1)
    }

    /// Checks if the Dimensions are same.
    fn same(d1: &Dimensions, d2: &Dimensions) -> bool {
        matches!(d1.cmp(d2), std::cmp::Ordering::Equal)
    }
}

/// Represents the Bound consisting of min/max width and min/max height.
#[derive(Debug, PartialEq)]
struct Bounds {
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

impl Bounds {
    /// Creates a new Bounds.
    fn new(min_width: u32, max_width: u32, min_height: u32, max_height: u32) -> Bounds {
        Bounds {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }
    /// Get the max bounds from the provided Dimensions (width & height).
    fn get_max_bounds(src: Dimensions, tgt: Dimensions) -> Result<Bounds, String> {
        let Dimensions(w1, h1) = src;
        let Dimensions(w2, h2) = tgt;

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

    /// Checks if the max bound (bounds.max_width * bounds.max_height) is greater than the parameter.
    fn is_greater_than(&self, other: u32) -> bool {
        (self.max_width * self.max_height) > other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_true_for_matching_dimensions() {
        let src = Dimensions(1, 1);
        let tgt = Dimensions(1, 1);

        assert!(Dimensions::same(&src, &tgt));
    }

    #[test]
    fn should_return_false_for_mismatching_dimensions() {
        let src = Dimensions(0, 0);
        let tgt = Dimensions(1, 1);

        assert!(!Dimensions::same(&src, &tgt));
    }

    #[test]
    fn should_return_zero_for_matching_images() {
        let src = image::ImageBuffer::new(100, 100);
        let tgt = image::ImageBuffer::new(100, 100);
        let bounds = Bounds::new(0, 100, 0, 100);

        assert_eq!(0, pixel_difference(&src, &tgt, &bounds));
    }

    #[test]
    fn should_return_non_zero_value_for_mismatching_images() {
        let src = image::ImageBuffer::new(100, 100);

        let mut tgt = image::ImageBuffer::new(100, 100);
        *tgt.get_pixel_mut(10, 10) = image::Rgba([10, 10, 10, 255]);
        *tgt.get_pixel_mut(20, 20) = image::Rgba([10, 10, 10, 255]);

        let bounds = Bounds::new(0, 100, 0, 100);

        assert_eq!(2, pixel_difference(&src, &tgt, &bounds));
    }

    #[test]
    fn should_return_ok_for_non_zero_bounds() {
        let src = Dimensions::from((10, 100));
        let tgt = Dimensions::from((100, 10));

        assert_eq!(
            Ok(Bounds::new(0, 10, 0, 10)),
            Bounds::get_max_bounds(src, tgt)
        );
    }

    #[test]
    fn should_return_err_for_zero_bounds() {
        let src = Dimensions::from((0, 0));
        let tgt = Dimensions::from((1, 1));

        assert_eq!(
            Err(String::from("Maximum width / height cannot be ZERO (0).")),
            Bounds::get_max_bounds(src, tgt)
        );
    }

    #[test]
    fn should_generate_name_from_backup_if_option_is_none() {
        assert_eq!(
            Some(PathBuf::from("/target_test_diff.png")),
            generate_output_file_name(None, &PathBuf::from("/target_test.png"))
        );
    }

    #[test]
    fn should_generate_name_from_option_if_option_is_some() {
        assert_eq!(
            Some(PathBuf::from("/custom_output_file.png")),
            generate_output_file_name(
                Some(String::from("custom_output_file")),
                &PathBuf::from("/target_test.png"),
            )
        );
    }

    #[test]
    pub fn should_return_zero_value_tuple_when_differences_are_observed() {
        let src = image::ImageBuffer::new(100, 100);
        let tgt = image::ImageBuffer::new(100, 100);

        let bounds = Bounds::new(0, 20, 0, 20);

        let (diff, bounds_with_diff) = percentage_difference(&src, &tgt, &bounds, 10);

        assert_eq!(0.0, diff);
        assert_eq!(Vec::<Bounds>::new(), bounds_with_diff);
    }

    #[test]
    pub fn should_return_non_zero_tuple_when_differences_are_observed() {
        let src = image::ImageBuffer::new(100, 100);

        let mut tgt = image::ImageBuffer::new(100, 100);
        *tgt.get_pixel_mut(15, 15) = image::Rgba([10, 10, 10, 255]);
        *tgt.get_pixel_mut(55, 55) = image::Rgba([10, 10, 10, 255]);

        let bounds = Bounds::new(0, 20, 0, 20);

        let (diff, bounds_with_diff) = percentage_difference(&src, &tgt, &bounds, 10);

        assert_eq!(0.25, diff);
        assert_eq!(vec![Bounds::new(10, 20, 10, 20)], bounds_with_diff);
    }

    #[test]
    pub fn should_highlight_only_the_specified_bounds() {
        let img = image::ImageBuffer::new(100, 100);

        let mut img_clone1 = img.clone();
        let bounds = vec![Bounds::new(10, 20, 10, 20), Bounds::new(50, 60, 50, 60)];
        highlight(&mut img_clone1, bounds);

        let mut img_clone2 = img.clone();
        for i in 10..20 {
            *img_clone2.get_pixel_mut(i, 10) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(i, 19) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(10, i) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(19, i) = image::Rgba([255, 0, 0, 255]);
        }
        for i in 50..60 {
            *img_clone2.get_pixel_mut(i, 50) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(i, 59) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(50, i) = image::Rgba([255, 0, 0, 255]);
            *img_clone2.get_pixel_mut(59, i) = image::Rgba([255, 0, 0, 255]);
        }

        assert_ne!(img, img_clone1);
        assert_eq!(img_clone2, img_clone1);
    }
}
