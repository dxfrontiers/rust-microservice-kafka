

use s3::bucket::Bucket;
use s3::credentials::Credentials;

use s3::region::Region;
use bytes::Bytes;
use crate::config::ConverterConfig;
use std::error::Error;
use crate::model::NetworkError;

/**
    Push a bunch of bytes (from an image) to s3
    We use a fresh HTTP client for every image, since the URLs might change for every received message
    Since the S3 library is a bit broken, we botch the error handling here
 */
pub async fn push_to_s3(bytes: Bytes, config: ConverterConfig, target_path: String)
    ->Result<(), Box<dyn Error + Send + Sync> > {

    // since the S3 library does not support uploading in a future out of the box, we wrap it in one
    // this is blocking and therefore spawned in the blocking thread pool of tokio
    tokio::task::spawn_blocking(move || {
        get_bucket_accessor(&config)
            .and_then(|bucket|
                bucket
                    .put_object(&target_path, &bytes, "image/jpg")
                    .map(|_| ()) // we don't need the result since it is broken anyway
                    .map_err(|e|
                        Box::new(NetworkError::UploadPutError(
                            e.description.unwrap_or("Generic upload error".into()).into())
                        ).into()
                    )
            )
    }).await?
}


fn get_bucket_accessor(config: &ConverterConfig) -> Result<Bucket, Box<dyn Error + Send + Sync> > {
    let bucket_name = &config.s3_bucket_name.clone();
    let aws_access = config.s3_access.clone();
    let aws_secret = config.s3_secret.clone();

    let region_name = config.s3_bucket_name.clone();
    let endpoint = config.s3_endpoint.clone();
    let region = Region::Custom { region: region_name, endpoint };

    let credentials: Credentials = Credentials::new(
        Some(aws_access),
        Some(aws_secret),
        None,
        None);

    // create the struct for the bucket
    Bucket::new(bucket_name, region, credentials)
        .map_err(|d|
            Box::new(NetworkError::UploadBucketAccessError(
                d.description.unwrap_or("Generic upload error".into()).into())
            ).into()
        )
}