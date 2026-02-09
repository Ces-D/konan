use aws_sdk_iotdataplane::primitives::Blob;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use lambda_shared::{IotConfigEnv, Message, create_iot_client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct PrintableMessage {
    content: tiptap::JSONContent,
    rows: Option<u32>,
}

async fn func(event: LambdaEvent<PrintableMessage>) -> Result<Message, Error> {
    let iot_env = IotConfigEnv::new();
    let client = create_iot_client(iot_env.endpoint).await;
    let payload = serde_json::to_string(&event.payload).unwrap();
    client
        .publish()
        .topic(iot_env.topic)
        .payload(Blob::new(payload))
        .qos(0)
        .send()
        .await?;
    Ok(Message::default())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}
