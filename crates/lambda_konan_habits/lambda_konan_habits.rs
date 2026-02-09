use aws_sdk_iotdataplane::primitives::Blob;
use chrono::{DateTime, Utc};
use lambda_http::{
    Error, IntoResponse, Request, RequestPayloadExt,
    http::{Response, StatusCode},
    run, service_fn,
};
use lambda_shared::{IotConfigEnv, Message, create_iot_client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct HabitTrackerTemplate {
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

async fn handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body = event.payload::<HabitTrackerTemplate>()?;
    tracing::info!("Received event body: {:?}", body);
    let iot_env = IotConfigEnv::new();
    let client = create_iot_client(iot_env.endpoint).await;
    let payload = serde_json::to_string(&body).unwrap();
    client
        .publish()
        .topic(iot_env.topic)
        .payload(Blob::new(payload))
        .qos(0)
        .send()
        .await?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::json!(Message::default()).to_string())
        .map_err(Box::new)?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_shared::initialize_tracing();
    run(service_fn(handler)).await
}
