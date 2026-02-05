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

/// Name of Konan MQTT topic
pub fn topic() -> String {
    std::env::var("KONAN_TOPIC").expect("KONAN_TOPIC must be set")
}
