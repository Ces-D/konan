use aws_config::BehaviorVersion;
use aws_sdk_iotdataplane::primitives::Blob;
use chrono::{DateTime, Utc};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use lambda_shared::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct HabitTrackerTemplate {
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

async fn func(event: LambdaEvent<HabitTrackerTemplate>) -> Result<Message, Error> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_iotdataplane::Client::new(&config);
    let payload = serde_json::to_string(&event.payload).unwrap();
    let blob = Blob::new(payload);
    client
        .publish()
        .topic(lambda_shared::topic())
        .payload(blob)
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
