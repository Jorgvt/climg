use image::{ImageBuffer, ImageFormat, Rgb};
use std::io::Write;
use std::process::{Command, Stdio};

fn create_sample_png(path: &std::path::Path, width: u32, height: u32) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        let r = (x % 256) as u8;
        let g = (y % 256) as u8;
        let b = ((x + y) % 256) as u8;
        Rgb([r, g, b])
    });
    img.save_with_format(path, ImageFormat::Png).unwrap();
}

fn create_sample_jpeg(path: &std::path::Path, width: u32, height: u32) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([(x % 256) as u8, (y % 256) as u8, 128])
    });
    img.save_with_format(path, ImageFormat::Jpeg).unwrap();
}

#[test]
fn test_cli_kitty_protocol() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|_| {
        let p = std::env::temp_dir().join("climg_test_kitty");
        std::fs::create_dir_all(&p).unwrap();
        tempfile::TempDir::new_in(std::env::temp_dir()).unwrap()
    });
    let img_path = temp_dir.path().join("plot.png");
    create_sample_png(&img_path, 200, 100);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img_path)
        .arg("--protocol")
        .arg("kitty")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plot.png"));
    assert!(stdout.contains("200x100"));
    assert!(stdout.contains("\x1b_Ga=T,f=100"));
}

#[test]
fn test_cli_iterm2_protocol() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img_path = temp_dir.path().join("metric.png");
    create_sample_png(&img_path, 150, 75);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img_path)
        .arg("--protocol")
        .arg("iterm2")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("metric.png"));
    assert!(stdout.contains("150x75"));
    assert!(stdout.contains("\x1b]1337;File="));
}

#[test]
fn test_cli_quiet_mode() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img_path = temp_dir.path().join("quiet.png");
    create_sample_png(&img_path, 64, 64);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img_path)
        .arg("--protocol")
        .arg("kitty")
        .arg("--quiet")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("quiet.png"));
    assert!(stdout.contains("\x1b_Ga=T,f=100"));
}

#[test]
fn test_cli_stdin_pipe() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img_path = temp_dir.path().join("piped.png");
    create_sample_png(&img_path, 80, 40);

    let img_bytes = std::fs::read(&img_path).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg("-")
        .arg("--protocol")
        .arg("kitty")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn climg");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(&img_bytes).unwrap();
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<stdin>"));
    assert!(stdout.contains("\x1b_Ga=T,f=100"));
}

#[test]
fn test_cli_jpeg_conversion() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img_path = temp_dir.path().join("photo.jpg");
    create_sample_jpeg(&img_path, 100, 100);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img_path)
        .arg("--protocol")
        .arg("kitty")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("photo.jpg"));
    assert!(stdout.contains("JPEG"));
    // Kitty converts JPEG to PNG payload
    assert!(stdout.contains("\x1b_Ga=T,f=100"));
}

#[test]
fn test_cli_multiple_files() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img1 = temp_dir.path().join("plot1.png");
    let img2 = temp_dir.path().join("plot2.png");
    create_sample_png(&img1, 50, 50);
    create_sample_png(&img2, 60, 60);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img1)
        .arg(&img2)
        .arg("--protocol")
        .arg("iterm2")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plot1.png"));
    assert!(stdout.contains("plot2.png"));
}
