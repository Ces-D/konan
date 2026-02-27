use aws_config::Region;
use aws_sdk_iotdataplane::Client;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
    pub message: String,
}
impl Default for Message {
    fn default() -> Self {
        Self {
            message: "Print command sent! Check rongta for print-out".to_string(),
        }
    }
}

pub struct IotConfigEnv {
    pub endpoint: String,
    pub topic: String,
}
impl Default for IotConfigEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl IotConfigEnv {
    pub fn new() -> Self {
        let endpoint = std::env::var("IOT_ENDPOINT").expect("IOT_ENDPOINT not set");
        let topic = std::env::var("IOT_TOPIC").expect("IOT_TOPIC not set");
        Self { endpoint, topic }
    }
}

pub async fn create_iot_client(endpoint: String) -> Client {
    let shared_config = aws_config::from_env()
        .region(Region::new("us-east-1"))
        .load()
        .await;
    let config = aws_sdk_iotdataplane::config::Builder::from(&shared_config)
        .endpoint_url(format!("https://{}", endpoint))
        .build();
    
    aws_sdk_iotdataplane::Client::from_conf(config)
}

pub fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // This creates a single-line output that CloudWatch likes
        .with_target(false)
        .without_time()
        .init();
}
