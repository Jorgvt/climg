use image::codecs::gif::GifEncoder;
use image::{Delay, Frame, ImageBuffer, ImageFormat, Rgb, Rgba};
use std::fs::File;
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

fn create_sample_animated_gif(path: &std::path::Path) {
    let file = File::create(path).unwrap();
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();

    let frame1_buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(30, 30, Rgba([255, 0, 0, 255]));
    let frame2_buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(30, 30, Rgba([0, 255, 0, 255]));

    let frame1 = Frame::from_parts(frame1_buf, 0, 0, Delay::from_numer_denom_ms(100, 1));
    let frame2 = Frame::from_parts(frame2_buf, 0, 0, Delay::from_numer_denom_ms(150, 1));

    encoder.encode_frame(frame1).unwrap();
    encoder.encode_frame(frame2).unwrap();
}

#[test]
fn test_cli_kitty_animated_gif() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let gif_path = temp_dir.path().join("anim.gif");
    create_sample_animated_gif(&gif_path);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&gif_path)
        .arg("--protocol")
        .arg("kitty")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("anim.gif"));
    assert!(stdout.contains("GIF"));
    // Frame 1: a=T,i=...,f=100,z=100
    assert!(stdout.contains("\x1b_Ga=T,i="));
    assert!(stdout.contains("z=100"));
    // Frame 2: a=f,i=...,f=100,z=150
    assert!(stdout.contains("\x1b_Ga=f,i="));
    assert!(stdout.contains("z=150"));
    // Playback control: a=a,i=...,s=3
    assert!(stdout.contains("\x1b_Ga=a,i="));
    assert!(stdout.contains("s=3"));
}

#[test]
fn test_cli_kitty_animated_gif_static_flag() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let gif_path = temp_dir.path().join("anim_static.gif");
    create_sample_animated_gif(&gif_path);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&gif_path)
        .arg("--protocol")
        .arg("kitty")
        .arg("--static")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("anim_static.gif"));
    // Should NOT contain multi-frame animation commands
    assert!(!stdout.contains("\x1b_Ga=f,i="));
    assert!(!stdout.contains("\x1b_Ga=a,i="));
    // Should contain standard single-frame transmit
    assert!(stdout.contains("\x1b_Ga=T,f=100"));
}

#[test]
fn test_cli_env_var_protocol() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let img_path = temp_dir.path().join("env_test.png");
    create_sample_png(&img_path, 80, 40);

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg(&img_path)
        .env("CLIMG_PROTOCOL", "iterm2")
        .env_remove("KITTY_WINDOW_ID")
        .env_remove("TERM_PROGRAM")
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b]1337;File="));
}

#[test]
fn test_cli_init_config() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_climg"))
        .arg("--init-config")
        .arg("kitty")
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("Failed to execute climg");

    assert!(output.status.success());
    let config_file = temp_dir.path().join("climg").join("config.toml");
    assert!(config_file.exists());
    let content = std::fs::read_to_string(config_file).unwrap();
    assert!(content.contains("protocol = \"kitty\""));
}
