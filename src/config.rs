use structopt::StructOpt;
use log::Level;

/**
    Obviously defaults are not the best idea here, so please remove them if you copy paste this code.
*/
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "downloader")]
pub struct ConverterConfig{
    #[structopt(short="r", long, default_value = "eu-east2", env = "S3_REGION_NAME")]
    pub s3_region_name: String,

    #[structopt(short, long, default_value = "http://127.0.0.1:9099")]
    pub s3_endpoint: String,

    #[structopt(short, long, default_value = "minioadmin")]
    pub s3_access: String,

    #[structopt(short, long, default_value = "minioadmin")]
    pub s3_secret: String,

    #[structopt(short, long, default_value = "foobar")]
    pub s3_bucket_name: String,

    #[structopt(short, long, default_value = "converter_tasks")]
    pub kafka_input_topic: String,

    #[structopt(short, long, default_value = "converter_results")]
    pub kafka_output_topic: String,

    #[structopt(short, long, default_value = "127.0.0.1:9092")]
    pub kafka_brokers: String,

    #[structopt(short, long, default_value = "converter_group")]
    pub kafka_consumer_group: String,

    #[structopt(short, long, default_value = "info")]
    pub log_level: Level,

    #[structopt(short, long, default_value = "5")]
    pub parallel_operations: usize,
}