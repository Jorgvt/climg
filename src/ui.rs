use crate::image_loader::LoadedImage;
use std::io::{self, Write};

pub struct Ui;

impl Ui {
    /// Print a styled metadata banner for the image
    pub fn print_banner<W: Write>(writer: &mut W, img: &LoadedImage) -> io::Result<()> {
        // Bold cyan name, subtle gray metadata
        writeln!(
            writer,
            "\x1b[1;36m▶ {}\x1b[0m \x1b[90m[{}x{} • {} • {}]\x1b[0m",
            img.name,
            img.width,
            img.height,
            img.format_name(),
            img.human_file_size()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_formatting() {
        let img = LoadedImage {
            name: "plot.png".to_string(),
            width: 800,
            height: 600,
            format: Some(image::ImageFormat::Png),
            file_size: 1024,
            raw_bytes: vec![],
        };
        let mut buf = Vec::new();
        Ui::print_banner(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("plot.png"));
        assert!(output.contains("800x600"));
        assert!(output.contains("PNG"));
        assert!(output.contains("1.0 KB"));
    }
}
