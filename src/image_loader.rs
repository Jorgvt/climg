use anyhow::{Context, Result, anyhow};
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageFormat};
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub png_bytes: Vec<u8>,
    pub delay_ms: u32,
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: Option<ImageFormat>,
    pub file_size: usize,
    pub raw_bytes: Vec<u8>,
}

impl LoadedImage {
    /// Load an image from a filesystem path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file: '{}'", path.display()))?;
        
        let mut raw_bytes = Vec::new();
        file.read_to_end(&mut raw_bytes)
            .with_context(|| format!("Failed to read file: '{}'", path.display()))?;

        if raw_bytes.is_empty() {
            return Err(anyhow!("File is empty: '{}'", path.display()));
        }

        Self::from_bytes(name, raw_bytes)
    }

    /// Load an image from standard input
    pub fn from_stdin() -> Result<Self> {
        let mut raw_bytes = Vec::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle
            .read_to_end(&mut raw_bytes)
            .context("Failed to read image data from stdin")?;

        if raw_bytes.is_empty() {
            return Err(anyhow!("No data received from stdin"));
        }

        Self::from_bytes("<stdin>".to_string(), raw_bytes)
    }

    /// Construct LoadedImage from raw bytes and inspect format & dimensions
    pub fn from_bytes(name: String, raw_bytes: Vec<u8>) -> Result<Self> {
        let file_size = raw_bytes.len();
        let reader = image::ImageReader::new(io::Cursor::new(&raw_bytes))
            .with_guessed_format()
            .context("Failed to detect image format")?;

        let format = reader.format();
        
        // Read dimensions (or decode full image if reader can't inspect dimensions alone)
        let (width, height) = match reader.into_dimensions() {
            Ok(dims) => dims,
            Err(_) => {
                let img = image::load_from_memory(&raw_bytes)
                    .context("Failed to decode image data")?;
                img.dimensions()
            }
        };

        Ok(Self {
            name,
            width,
            height,
            format,
            file_size,
            raw_bytes,
        })
    }

    /// Decode to a DynamicImage if pixel manipulation or re-encoding is needed
    pub fn to_dynamic_image(&self) -> Result<DynamicImage> {
        image::load_from_memory(&self.raw_bytes)
            .context("Failed to decode image to DynamicImage")
    }

    /// Extract animation frames if this image is an animated format (e.g. GIF) with > 1 frames
    pub fn extract_animation_frames(&self) -> Result<Option<Vec<AnimationFrame>>> {
        if self.format != Some(ImageFormat::Gif) {
            return Ok(None);
        }

        let decoder = match GifDecoder::new(Cursor::new(&self.raw_bytes)) {
            Ok(d) => d,
            Err(_) => return Ok(None),
        };

        let frames = decoder.into_frames().collect_frames()?;
        if frames.len() <= 1 {
            return Ok(None);
        }

        let mut anim_frames = Vec::with_capacity(frames.len());
        for frame in frames {
            let delay = frame.delay();
            let (num, denom) = delay.numer_denom_ms();
            let delay_ms = if denom > 0 {
                num / denom
            } else {
                num
            };
            // Default to at least 10ms if delay is 0
            let delay_ms = if delay_ms == 0 { 100 } else { delay_ms };

            let buf = frame.into_buffer();
            let dyn_img = DynamicImage::ImageRgba8(buf);
            let mut png_bytes = Vec::new();
            dyn_img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;

            anim_frames.push(AnimationFrame {
                png_bytes,
                delay_ms,
            });
        }

        Ok(Some(anim_frames))
    }

    /// Human-readable format name
    pub fn format_name(&self) -> &str {
        match self.format {
            Some(ImageFormat::Png) => "PNG",
            Some(ImageFormat::Jpeg) => "JPEG",
            Some(ImageFormat::WebP) => "WebP",
            Some(ImageFormat::Gif) => "GIF",
            Some(other) => other.extensions_str().first().copied().unwrap_or("Unknown"),
            None => "Unknown",
        }
    }

    /// Human-readable file size (e.g. "124.5 KB")
    pub fn human_file_size(&self) -> String {
        let size = self.file_size as f64;
        if size < 1024.0 {
            format!("{} B", size as usize)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_png_bytes() {
        // 1x1 red PNG
        let png_bytes = [
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00,
            0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
            0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let loaded = LoadedImage::from_bytes("test.png".to_string(), png_bytes.to_vec()).unwrap();
        assert_eq!(loaded.width, 1);
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.format_name(), "PNG");
    }
}
