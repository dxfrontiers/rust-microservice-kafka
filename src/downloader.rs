use bytes::{BytesMut, Bytes};
use std::error::Error;
use log::debug;
use crate::model::NetworkError;

pub async fn download_url(url: String) -> Result<Bytes, Box<dyn Error + Send + Sync> >{

    let mut body = reqwest::get(&url).await?;

    let code = body.status();

    match code.as_u16() {
        200 => {
            let mut res = BytesMut::new();
            let mut total_len = 0;
            while let Some(chunk) = body.chunk().await? {
                total_len+=chunk.len();
                if  total_len > 20*1024*1024{
                    return Err(Box::new(NetworkError::DownloadSizeLimitReached(total_len as u16)));
                }
                res.extend_from_slice(&chunk);
            }
            debug!("Downloaded data of size: {} bytes",total_len);
            let body = res.freeze();
            Ok(body)
        }
        _ => {
            Err(NetworkError::DownloadError(code.canonical_reason().unwrap_or("Unknown HTTP error").into()).into())
        }
    }


}