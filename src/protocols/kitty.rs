use anyhow::Result;
use base64::prelude::*;
use std::io::Write;

const CHUNK_SIZE: usize = 4096;

pub struct KittyEncoder;

impl KittyEncoder {
    /// Format escape commands for Kitty graphics protocol using PNG payload (`f=100`).
    /// If `wrap_fn` is provided, each escape chunk is wrapped (e.g. for tmux passthrough).
    pub fn build_chunks(
        png_bytes: &[u8],
        cols: Option<u32>,
        rows: Option<u32>,
    ) -> Vec<String> {
        let b64 = BASE64_STANDARD.encode(png_bytes);
        let b64_bytes = b64.as_bytes();
        let mut chunks = Vec::new();

        let total_chunks = (b64_bytes.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        if total_chunks == 0 {
            return chunks;
        }

        // Build sizing flags
        let mut size_flags = String::new();
        if let Some(c) = cols {
            size_flags.push_str(&format!(",c={}", c));
        }
        if let Some(r) = rows {
            size_flags.push_str(&format!(",r={}", r));
        }

        for (i, slice) in b64_bytes.chunks(CHUNK_SIZE).enumerate() {
            let chunk_str = std::str::from_utf8(slice).unwrap_or("");
            let is_first = i == 0;
            let is_last = i == total_chunks - 1;

            let m = if is_last { 0 } else { 1 };

            let seq = if is_first {
                if is_last {
                    format!("\x1b_Ga=T,f=100{};{}\x1b\\", size_flags, chunk_str)
                } else {
                    format!("\x1b_Ga=T,f=100,m=1{};{}\x1b\\", size_flags, chunk_str)
                }
            } else {
                format!("\x1b_Gm={};{}\x1b\\", m, chunk_str)
            };

            chunks.push(seq);
        }

        chunks
    }

    /// Write all chunks to the output stream, applying `wrap_fn` to each chunk.
    pub fn write_png<W: Write, F: Fn(&str) -> String>(
        writer: &mut W,
        png_bytes: &[u8],
        cols: Option<u32>,
        rows: Option<u32>,
        wrap_fn: F,
    ) -> Result<()> {
        let chunks = Self::build_chunks(png_bytes, cols, rows);
        for chunk in chunks {
            let wrapped = wrap_fn(&chunk);
            writer.write_all(wrapped.as_bytes())?;
        }
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitty_single_chunk() {
        let data = b"hello world";
        let chunks = KittyEncoder::build_chunks(data, Some(40), Some(20));
        assert_eq!(chunks.len(), 1);
        let b64 = BASE64_STANDARD.encode(data);
        let expected = format!("\x1b_Ga=T,f=100,c=40,r=20;{}\x1b\\", b64);
        assert_eq!(chunks[0], expected);
    }

    #[test]
    fn test_kitty_multiple_chunks() {
        // Data larger than 4096 bytes encoded (4096 / 4 * 3 = 3072 bytes raw)
        let large_data = vec![0u8; 6000];
        let chunks = KittyEncoder::build_chunks(&large_data, None, None);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].starts_with("\x1b_Ga=T,f=100,m=1;"));
        assert!(chunks[chunks.len() - 1].starts_with("\x1b_Gm=0;"));
    }
}
