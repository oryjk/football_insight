use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ParsedAvatar {
    pub bytes: Vec<u8>,
    pub mime_type: &'static str,
    pub ext: &'static str,
}

#[async_trait]
pub trait AvatarStoragePort: Send + Sync {
    fn max_bytes(&self) -> usize;

    async fn store_avatar(
        &self,
        user_id: Uuid,
        parsed: ParsedAvatar,
    ) -> anyhow::Result<String>;
}

pub fn parse_avatar_data_url(data_url: &str, max_bytes: usize) -> anyhow::Result<ParsedAvatar> {
    let trimmed = data_url.trim();
    if trimmed.is_empty() {
        anyhow::bail!("avatar data url is required");
    }
    let Some((header, b64)) = trimmed.split_once(',') else {
        anyhow::bail!("avatar data url is invalid");
    };
    let header_lower = header.trim().to_ascii_lowercase();
    let is_supported_header = matches!(
        header_lower.as_str(),
        "data:image/jpeg;base64"
            | "data:image/jpg;base64"
            | "data:image/png;base64"
            | "data:image/webp;base64"
    );
    if !is_supported_header {
        anyhow::bail!("avatar data url is invalid");
    }

    use base64::{Engine, engine::general_purpose::STANDARD};
    let cleaned_b64: String = b64.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let bytes = STANDARD
        .decode(cleaned_b64)
        .map_err(|_| anyhow::anyhow!("avatar data url is invalid"))?;
    if bytes.is_empty() {
        anyhow::bail!("avatar data url is invalid");
    }
    if bytes.len() > max_bytes {
        anyhow::bail!("avatar image is too large");
    }

    let (mime_type, ext) = if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        ("image/jpeg", "jpg")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ("image/png", "png")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        ("image/webp", "webp")
    } else {
        anyhow::bail!("avatar image format is unsupported");
    };

    Ok(ParsedAvatar {
        bytes,
        mime_type,
        ext,
    })
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::parse_avatar_data_url;

    #[test]
    fn parses_valid_jpeg_and_png_magic_bytes() {
        let jpeg_b64 = STANDARD.encode([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10]);
        let parsed = parse_avatar_data_url(&format!("data:image/jpeg;base64,{jpeg_b64}"), 1024)
            .expect("jpeg should parse");
        assert_eq!(parsed.mime_type, "image/jpeg");
        assert_eq!(parsed.ext, "jpg");

        let png_b64 = STANDARD.encode(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR");
        let parsed = parse_avatar_data_url(&format!("data:image/png;base64,{png_b64}"), 1024)
            .expect("png should parse");
        assert_eq!(parsed.mime_type, "image/png");
        assert_eq!(parsed.ext, "png");
    }

    #[test]
    fn rejects_spoofed_magic_bytes_and_oversized_images() {
        let fake_b64 = STANDARD.encode(b"<svg onload=alert(1)>");
        assert!(parse_avatar_data_url(&format!("data:image/png;base64,{fake_b64}"), 1024).is_err());

        let big = STANDARD.encode(vec![0xff, 0xd8, 0xff, 0xe0, 0x01, 0x02]);
        assert!(parse_avatar_data_url(&format!("data:image/jpeg;base64,{big}"), 4).is_err());
    }
}
