use tokio::runtime::Runtime;

use structopt::StructOpt;
use log::info;

mod config;
mod kafka;
mod downloader;
mod image_processing;
mod s3_publisher;
mod model;

use config::ConverterConfig;

use futures::future::join_all;


fn main() {
    
    let cfg = ConverterConfig::from_args();

    simple_logger::init_with_level(cfg.log_level).expect("Could not init logger");
    info!("Logging initialized with level {}", cfg.log_level);

    // Spawn a set of workers.
    // This has to be in an async task to have access to the `tokio::spawn` primitive
    Runtime::new().unwrap().block_on(async {
        let futures =
            (0..cfg.parallel_operations)
            .map(|_| {
                tokio::spawn(
                    kafka::receive_messages(cfg.clone())
                )
            });

        // wait for all workers to finish
        join_all(futures).await;
    });
}
