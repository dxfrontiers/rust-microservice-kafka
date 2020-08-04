use bytes::{Bytes};
use std::error::Error;
use std::io::Cursor;
use image::jpeg::JPEGDecoder;
use image::DynamicImage;
use image::ImageFormat::JPEG;

/**
    Process the image, generate a thumbnail and return both, the original and the thumbnail
*/
pub fn process_image(input_bytes: Bytes) -> Result<(Bytes,Bytes), Box<dyn Error + Send + Sync> >{

    let full_size_image = input_bytes.clone();
    let cursor = Cursor::new(input_bytes);

    let dec = JPEGDecoder::new(cursor)?;
    let di = DynamicImage::from_decoder(dec)?;

    let processed = di.thumbnail(128,128);

    let mut v = Vec::new();
    processed.write_to(&mut v,JPEG)?;

    Ok((full_size_image,Bytes::copy_from_slice(&v)))
}
