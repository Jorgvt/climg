use crate::cli::Cli;
use crate::image_loader::LoadedImage;
use crate::protocols::iterm2::Iterm2Encoder;
use crate::protocols::kitty::KittyEncoder;
use crate::terminal::{GraphicsProtocol, TerminalInfo};
use anyhow::{Context, Result, anyhow};
use image::ImageFormat;
use std::io::{Cursor, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeSpec {
    Cells(u32),
    Percent(f32),
}

impl SizeSpec {
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if let Some(percent_str) = s.strip_suffix('%') {
            let pct: f32 = percent_str
                .parse()
                .map_err(|_| anyhow!("Invalid percentage: '{}'", s))?;
            if pct <= 0.0 || pct > 1000.0 {
                return Err(anyhow!("Percentage out of reasonable range: '{}'", s));
            }
            Ok(SizeSpec::Percent(pct / 100.0))
        } else if let Some(px_str) = s.strip_suffix("px") {
            // Rough conversion from pixels to cells (~8px per col width, ~16px per row height)
            let px: u32 = px_str
                .parse()
                .map_err(|_| anyhow!("Invalid pixel value: '{}'", s))?;
            // Return as cells approximation (8px per col)
            Ok(SizeSpec::Cells((px / 8).max(1)))
        } else {
            let cells: u32 = s
                .parse()
                .map_err(|_| anyhow!("Invalid cell count: '{}'", s))?;
            if cells == 0 {
                return Err(anyhow!("Cell count must be greater than 0: '{}'", s));
            }
            Ok(SizeSpec::Cells(cells))
        }
    }

    pub fn to_cells(&self, total_cells: u16) -> u32 {
        match self {
            SizeSpec::Cells(c) => *c,
            SizeSpec::Percent(p) => ((total_cells as f32) * p).round().max(1.0) as u32,
        }
    }
}

pub struct RenderOptions {
    pub width: Option<SizeSpec>,
    pub height: Option<SizeSpec>,
    pub max_width: Option<SizeSpec>,
    pub max_height: Option<SizeSpec>,
}

impl RenderOptions {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        let width = cli.width.as_deref().map(SizeSpec::parse).transpose()?;
        let height = cli.height.as_deref().map(SizeSpec::parse).transpose()?;
        let max_width = cli.max_width.as_deref().map(SizeSpec::parse).transpose()?;
        let max_height = cli.max_height.as_deref().map(SizeSpec::parse).transpose()?;

        Ok(Self {
            width,
            height,
            max_width,
            max_height,
        })
    }
}

pub struct Renderer;

impl Renderer {
    /// Calculate effective target columns and rows based on terminal window and flags
    pub fn compute_target_dimensions(
        opts: &RenderOptions,
        term_cols: u16,
        term_rows: u16,
    ) -> (Option<u32>, Option<u32>) {
        let max_w = opts
            .max_width
            .map(|s| s.to_cells(term_cols))
            .unwrap_or(term_cols as u32);

        // Leave 2-3 lines for prompt/status if no height constraint specified
        let default_max_h = (term_rows.saturating_sub(3)).max(10) as u32;
        let max_h = opts
            .max_height
            .map(|s| s.to_cells(term_rows))
            .unwrap_or(default_max_h);

        let mut target_cols = match opts.width {
            Some(w) => Some(w.to_cells(term_cols).min(max_w)),
            None => None,
        };

        let mut target_rows = match opts.height {
            Some(h) => Some(h.to_cells(term_rows).min(max_h)),
            None => None,
        };

        // If neither width nor height is explicitly given, constrain to terminal bounds
        if target_cols.is_none() && target_rows.is_none() {
            target_cols = Some(max_w);
            target_rows = Some(max_h);
        }

        (target_cols, target_rows)
    }

    /// Render loaded image to writer using terminal info and render options
    pub fn render<W: Write>(
        writer: &mut W,
        img: &LoadedImage,
        opts: &RenderOptions,
        term_info: &TerminalInfo,
    ) -> Result<()> {
        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let (cols, rows) = Self::compute_target_dimensions(opts, term_cols, term_rows);

        let wrap_fn = |seq: &str| term_info.wrap_escape(seq);

        match term_info.protocol {
            GraphicsProtocol::Kitty => {
                // If already PNG, send directly; otherwise transcode to PNG in memory
                let png_bytes = if img.format == Some(ImageFormat::Png) {
                    img.raw_bytes.clone()
                } else {
                    let dyn_img = img.to_dynamic_image()?;
                    let mut buf = Vec::new();
                    dyn_img
                        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
                        .context("Failed to transcode image to PNG for Kitty protocol")?;
                    buf
                };

                KittyEncoder::write_png(writer, &png_bytes, cols, rows, wrap_fn)?;
            }
            GraphicsProtocol::Iterm2 => {
                // iTerm2 natively understands PNG, JPEG, GIF, and WebP directly
                let payload_bytes = if img.format.is_some() {
                    img.raw_bytes.clone()
                } else {
                    let dyn_img = img.to_dynamic_image()?;
                    let mut buf = Vec::new();
                    dyn_img
                        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
                        .context("Failed to encode image to PNG for iTerm2 protocol")?;
                    buf
                };

                Iterm2Encoder::write_image(
                    writer,
                    &payload_bytes,
                    cols,
                    rows,
                    Some(&img.name),
                    wrap_fn,
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_spec_parse() {
        assert_eq!(SizeSpec::parse("80").unwrap(), SizeSpec::Cells(80));
        assert_eq!(SizeSpec::parse("50%").unwrap(), SizeSpec::Percent(0.5));
        assert_eq!(SizeSpec::parse("800px").unwrap(), SizeSpec::Cells(100));
        assert!(SizeSpec::parse("0").is_err());
        assert!(SizeSpec::parse("abc").is_err());
    }

    #[test]
    fn test_compute_target_dimensions_default() {
        let opts = RenderOptions {
            width: None,
            height: None,
            max_width: None,
            max_height: None,
        };
        let (cols, rows) = Renderer::compute_target_dimensions(&opts, 120, 40);
        assert_eq!(cols, Some(120));
        assert_eq!(rows, Some(37));
    }

    #[test]
    fn test_compute_target_dimensions_explicit() {
        let opts = RenderOptions {
            width: Some(SizeSpec::Cells(60)),
            height: None,
            max_width: None,
            max_height: None,
        };
        let (cols, rows) = Renderer::compute_target_dimensions(&opts, 120, 40);
        assert_eq!(cols, Some(60));
        assert_eq!(rows, None);
    }
}
