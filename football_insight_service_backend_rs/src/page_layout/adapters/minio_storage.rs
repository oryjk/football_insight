use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    config::MinioConfig,
    page_layout::{
        domain::{PageLayoutError, ParsedLayoutImage},
        ports::LayoutImageStoragePort,
    },
};

pub struct MinioLayoutImageStoragePort {
    config: Option<MinioConfig>,
    client: Client,
}

impl MinioLayoutImageStoragePort {
    pub fn new(config: Option<MinioConfig>) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LayoutImageStoragePort for MinioLayoutImageStoragePort {
    async fn upload_image(&self, image: ParsedLayoutImage) -> Result<String, PageLayoutError> {
        let Some(config) = self.config.as_ref() else {
            return Err(PageLayoutError::ImageStorageUnavailable);
        };

        let object_key = format!("page-layouts/{}.{}", Uuid::new_v4(), image.ext);
        let public_url = format!(
            "{}/{}",
            config.public_base_url.trim_end_matches('/'),
            object_key
        );
        let endpoint = config.endpoint.trim_end_matches('/');
        let object_path = format!(
            "/{}/{}",
            percent_encode_path_segment(&config.bucket),
            object_key
                .split('/')
                .map(percent_encode_path_segment)
                .collect::<Vec<_>>()
                .join("/")
        );
        let url = format!("{endpoint}{object_path}");
        let payload_hash = hex_sha256(&image.bytes);
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let parsed_endpoint = reqwest::Url::parse(endpoint).map_err(|error| {
            tracing::warn!(error = %error, "invalid MinIO endpoint");
            PageLayoutError::Internal
        })?;
        let endpoint_host = parsed_endpoint.host_str().ok_or_else(|| {
            tracing::warn!("invalid MinIO endpoint host");
            PageLayoutError::Internal
        })?;
        let host = match parsed_endpoint.port() {
            Some(port) => format!("{endpoint_host}:{port}"),
            None => endpoint_host.to_string(),
        };
        let canonical_headers =
            format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n");
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request =
            format!("PUT\n{object_path}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}");
        let credential_scope = format!("{date_stamp}/{}/s3/aws4_request", config.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            hex_sha256(canonical_request.as_bytes())
        );
        let signing_key = signing_key(&config.secret_key, &date_stamp, &config.region);
        let signature = hex_hmac(&signing_key, string_to_sign.as_bytes());
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
            config.access_key
        );

        let response = self
            .client
            .put(url)
            .header("Host", host)
            .header("x-amz-date", amz_date)
            .header("x-amz-content-sha256", payload_hash)
            .header("Authorization", authorization)
            .header("Content-Type", image.mime_type)
            .body(image.bytes)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(error = %error, "failed to upload page layout image to MinIO");
                PageLayoutError::Internal
            })?;

        if let Err(error) = response.error_for_status() {
            tracing::warn!(error = %error, "MinIO rejected page layout image upload");
            return Err(PageLayoutError::Internal);
        }

        Ok(public_url)
    }
}

type HmacSha256 = Hmac<Sha256>;

fn signing_key(secret_key: &str, date_stamp: &str, region: &str) -> Vec<u8> {
    let date_key = hmac_sha256(
        format!("AWS4{secret_key}").as_bytes(),
        date_stamp.as_bytes(),
    );
    let date_region_key = hmac_sha256(&date_key, region.as_bytes());
    let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
    hmac_sha256(&date_region_service_key, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hex_hmac(key: &[u8], data: &[u8]) -> String {
    hex_encode(&hmac_sha256(key, data))
}

fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn percent_encode_path_segment(segment: &str) -> String {
    url::form_urlencoded::byte_serialize(segment.as_bytes()).collect()
}
