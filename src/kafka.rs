use std::error::Error;
use std::convert::TryFrom;

use rdkafka::consumer::{StreamConsumer};
use rdkafka::{ClientConfig, Message};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::Consumer;
use rdkafka::producer::{FutureProducer, FutureRecord};



use tokio::stream::StreamExt;
use log::{debug,info,error};

use crate::downloader::download_url;
use crate::image_processing::process_image;
use crate::config::ConverterConfig;
use crate::s3_publisher::push_to_s3;
use crate::model::{DownloadTask, DownloadResult, ProcessingResult};




/**
    Handling each and every message
    Note the stacked matches: We need to commit the message whenever processing finished (Ok or err)
*/
pub async fn receive_messages(cfg: ConverterConfig){

    let consumer = create_consumer(
        &cfg.kafka_brokers,
        &cfg.kafka_consumer_group,
        &cfg.kafka_input_topic);

    let producer = create_producer(&cfg.kafka_brokers);
    let output_topic = cfg.kafka_output_topic.to_owned();

    let mut msg_stream = consumer.start();

    // iterate over all messages blocking
    while let Some(msg) = msg_stream.next().await{
        // we cant borrow cfg to the async function, since that would require static a lifetime
        // therefore, we copy it (meh...)
        let cfg = cfg.clone();


        // the message itself can be broken
        match msg {
            Ok(msg) => {

                // tha payload can be empty
                match  msg.payload() {
                    Some(payload) => {
                        let request = DownloadTask::try_from(payload);

                        // only process valid messages
                        match request {
                            Ok(request) => {
                                handle_received_message(cfg, &producer, &output_topic, request).await;
                            }
                            Err(e) => {
                                error!("Error parsing payload: {}",e);
                            }
                        }
                    },
                    None => {
                        error!("Message with empty payload");
                    }
                }

                // now we can store the offset to be committed in the next auto-commit so this
                // message will never be processed again
                let res = consumer.store_offset(&msg);
                match res{
                    Ok(()) => {}
                    Err(e) => error!("Could not commit message: {} ", e)
                }
            }
            Err(e)=>{
                error!("Could not receive and will not process message: {}",e)
            }
        };
    }
}

async fn handle_received_message(cfg: ConverterConfig, producer: &FutureProducer, output_topic: &String, request: DownloadTask) {

    let correlation_id = request.correlation_id;
    let handle_result = process(request, cfg).await;

    let (res, description) = match handle_result {
        Ok(_) => (ProcessingResult::Ok, String::from("")),
        Err(e) => (ProcessingResult::Error, format!("{}", e)),
    };

    let dr = DownloadResult {
        correlation_id: correlation_id,
        res,
        description
    };

    // if the result can not be serialized, we rather send an empty string than to crash
    let dr_str_rep = serde_json::to_string(&dr).unwrap_or("".into());

    info!("Answer: {}", dr_str_rep);

    // this sends the "reply" to the other topic
    let res =
        send_ready_notification(&producer, output_topic.clone(), dr_str_rep).await;
    if res.is_err(){error!("Error sending ack: {}", res.err().expect("Fatal: could not send ack"))}
}


pub async fn process(request: DownloadTask, cfg: ConverterConfig)
                     -> Result<(), Box<dyn Error + Send + Sync> >{

    let bytes = download_url(request.url.into()).await?;

    let both_bytes = tokio::task::spawn_blocking(|| process_image(bytes)).await??;

    push_to_s3(both_bytes.0,cfg.clone(), request.target_path.clone()).await?;
    push_to_s3(both_bytes.1,cfg.clone(), request.target_thumbnail_path.clone()).await?;

    info!("Finished Download for {}",request.target_path);

    Ok(())
}


pub async fn send_ready_notification(producer: &FutureProducer, topic: String, record: String)
                                     -> Result<(), Box<dyn Error + Send + Sync> >{

    let record = FutureRecord::to(&topic)
        .key("some key".into())
        .payload(&record);

    let produce_future = producer.send(record,0);
    match produce_future.await {
        Ok(Ok(delivery)) => debug!("Sent: {:?}", delivery),
        Ok(Err((e, _))) => error!("Error: {:?}", e),
        Err(_) => error!("Future cancelled"),
    }
    Ok(())
}



pub fn create_producer(brokers: &str, ) -> FutureProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");
    producer
}


pub fn create_consumer(brokers: &str, group_id: &str, topic: &str) -> StreamConsumer {

    let consumer: StreamConsumer  = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000")
        .set("enable.auto.offset.store", "false")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topic");

    consumer
}

