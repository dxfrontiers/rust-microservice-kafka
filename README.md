# rust-microservice-kafka 

This is the source code for our [Blog Post](https://medium.com/digitalfrontiers/microservices-in-rust-with-kafka-2b671295b24e).

Although the service is rather incomplete, you might find some of the implemented patterns useful.

To run it yourself, you need to have a Kafka and S3-compatible object storgae available. These can be configured via commandline parameters.
To test image processing, the management service can be connected to kafka to create download tasks.
