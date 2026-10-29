use anyhow::Result;
use base64::prelude::*;
use std::io::Write;

pub struct Iterm2Encoder;

impl Iterm2Encoder {
    /// Build OSC 1337 inline file escape sequence
    pub fn build_sequence(
        image_bytes: &[u8],
        cols: Option<u32>,
        rows: Option<u32>,
        filename: Option<&str>,
    ) -> String {
        let b64 = BASE64_STANDARD.encode(image_bytes);

        let mut args = vec!["inline=1".to_string()];

        args.push(format!("size={}", image_bytes.len()));

        if let Some(name) = filename {
            let name_b64 = BASE64_STANDARD.encode(name.as_bytes());
            args.push(format!("name={}", name_b64));
        }

        match (cols, rows) {
            (Some(c), Some(r)) => {
                args.push(format!("width={}", c));
                args.push(format!("height={}", r));
            }
            (Some(c), None) => {
                args.push(format!("width={}", c));
                args.push("height=auto".to_string());
            }
            (None, Some(r)) => {
                args.push("width=auto".to_string());
                args.push(format!("height={}", r));
            }
            (None, None) => {
                args.push("width=auto".to_string());
                args.push("height=auto".to_string());
            }
        }

        args.push("preserveAspectRatio=1".to_string());

        let args_str = args.join(";");
        format!("\x1b]1337;File={}:{}\x07", args_str, b64)
    }

    /// Write iTerm2 inline image sequence to writer, wrapped with `wrap_fn`
    pub fn write_image<W: Write, F: Fn(&str) -> String>(
        writer: &mut W,
        image_bytes: &[u8],
        cols: Option<u32>,
        rows: Option<u32>,
        filename: Option<&str>,
        wrap_fn: F,
    ) -> Result<()> {
        let seq = Self::build_sequence(image_bytes, cols, rows, filename);
        let wrapped = wrap_fn(&seq);
        writer.write_all(wrapped.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterm2_sequence_generation() {
        let payload = b"test payload";
        let seq = Iterm2Encoder::build_sequence(payload, Some(80), Some(40), Some("loss.png"));
        assert!(seq.starts_with("\x1b]1337;File="));
        assert!(seq.ends_with("\x07"));
        assert!(seq.contains("inline=1"));
        assert!(seq.contains("width=80"));
        assert!(seq.contains("height=40"));
        assert!(seq.contains("preserveAspectRatio=1"));
        let b64 = BASE64_STANDARD.encode(payload);
        assert!(seq.contains(&b64));
    }

    #[test]
    fn test_iterm2_auto_dimensions() {
        let payload = b"auto test";
        let seq = Iterm2Encoder::build_sequence(payload, None, None, None);
        assert!(seq.contains("width=auto"));
        assert!(seq.contains("height=auto"));
    }
}
