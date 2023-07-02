use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn insta_test_help_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("idiff")?;
    command.arg("--help");

    let assert = command.assert().success();
    let output = std::str::from_utf8(&assert.get_output().stdout)?;

    insta::assert_snapshot!(&output, @r###"
    diff - for images (compares images pixel by pixel)

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
    "###);

    Ok(())
}

#[test]
fn should_fail_when_invalid_file_is_used() -> Result<(), Box<dyn std::error::Error>> {
    let err_msg = "Invalid values for src/tgt path. Please check and try again.";

    let temp_dir = assert_fs::TempDir::new()?;
    let temp_file = temp_dir.child("foo.png");
    temp_file.touch().unwrap();

    let mut command = Command::cargo_bin("idiff")?;
    command
        .arg("--src")
        .arg("/invalid/file/name")
        .arg("--tgt")
        .arg(temp_file.as_os_str());
    command
        .assert()
        .failure()
        .stderr(predicate::str::contains(err_msg));

    temp_dir.close()?;
    Ok(())
}

#[test]
fn should_fail_when_opening_invalid_file_as_image() -> Result<(), Box<dyn std::error::Error>> {
    let err_msg = "Encountered error while opening source / target image.";

    let temp_dir = assert_fs::TempDir::new()?;
    let temp_file = temp_dir.child("foo.png");
    temp_file.touch().unwrap();

    let mut command = Command::cargo_bin("idiff")?;
    command
        .arg("--src")
        .arg(temp_file.as_os_str())
        .arg("--tgt")
        .arg(temp_file.as_os_str());
    command
        .assert()
        .failure()
        .stderr(predicate::str::contains(err_msg));

    temp_dir.close()?;
    Ok(())
}
