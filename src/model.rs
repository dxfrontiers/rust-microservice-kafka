use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;
use std::fmt;

/**
    A download task as used with the management application
    We need to keep the field names (or at least what they will be serialized to), in sync with
    the version in the management service.
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadTask{
    pub correlation_id: u32,
    pub url: String,
    pub target_path: String,
    pub target_thumbnail_path: String
}

impl TryFrom<&[u8]> for DownloadTask{
    type Error = Box<dyn Error + Send + Sync> ;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
            // we allow rust to loose the type, so the next line is equivalent to this:
            //.map_err(|e|Box::new(e ) as Box<dyn Error + Send + Sync>)
            .map_err(|e|e.into())
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResult{
    pub correlation_id: u32,
    pub res: ProcessingResult,
    pub description: String
}

#[derive(Debug)]
pub enum NetworkError {
    DownloadSizeLimitReached(u16),
    DownloadError(String),
    UploadBucketAccessError(String),
    UploadPutError(String)
}


impl Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            NetworkError::DownloadSizeLimitReached(size) => write!(f, "DownloadSizeLimitReached: {}",size),
            NetworkError::DownloadError(msg) => write!(f, "DownloadError: {}",msg),
            NetworkError::UploadBucketAccessError(msg) => {write!(f, "UploadBucketAccessError: {}",msg)},
            NetworkError::UploadPutError(msg) => {write!(f, "UploadPutError: {}",msg)}
        }
    }
}
impl Error for NetworkError {}


#[derive(Serialize, Deserialize, Debug)]
pub enum ProcessingResult {
    Ok,
    Error
}