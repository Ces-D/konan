use anyhow::Result;
use chrono::{DateTime, Utc};
use designs::{
    box_template::BoxTemplateBuilder, habit_tracker_template::HabitTrackerTemplateBuilder,
    tiptap_interpreter::TipTapInterpreter,
};
use rongta::RongtaPrinter;
use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    sync::Arc,
};
use tokio::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct OutlineTemplate {
    rows: Option<u32>,
    date: Option<DateTime<Utc>>,
    banner: Option<String>,
    lined: Option<bool>,
}
#[derive(Debug, Deserialize, Serialize)]
struct PrintableMessage {
    content: tiptap::JSONContent,
    rows: Option<u32>,
}
#[derive(Debug, Deserialize, Serialize)]
struct HabitTrackerTemplate {
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting Konan iot client");
    // AWS IoT Core endpoint (replace with your endpoint)
    let endpoint = std::env::var("KONAN_IOT_ENDPOINT_URL").unwrap();
    let port = std::env::var("KONAN_IOT_PORT").unwrap().parse().unwrap();
    let client_id = std::env::var("KONAN_IOT_CLIENT_ID").unwrap();

    // Set up MQTT options
    let mut mqttoptions = MqttOptions::new(client_id, endpoint, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    // Configure TLS
    let tls_config = configure_tls(
        "~/.iot-device/certs/konan.pem",
        "~/.iot-device/certs/konan_private.key",
        "~/.iot-device/certs/AmazonRootCA1.pem",
    )?;

    mqttoptions.set_transport(Transport::Tls(tls_config));

    // Create client and event loop
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to a topic
    client
        .subscribe("command/konan_pi/habits", QoS::AtLeastOnce)
        .await?;
    log::info!("Subscribed to `command/konan_pi/habits`");
    client
        .subscribe("command/konan_pi/message", QoS::AtLeastOnce)
        .await?;
    log::info!("Subscribed to `command/konan_pi/message`");
    client
        .subscribe("command/konan_pi/outline", QoS::AtLeastOnce)
        .await?;
    log::info!("Subscribed to `command/konan_pi/outline`");

    // Handle incoming messages
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                    let payload = String::from_utf8_lossy(&msg.payload);
                    log::trace!("Received message on topic '{}': {}", msg.topic, payload);
                    let builder = RongtaPrinter::new(true);
                    let pattern = designs::get_random_box_pattern()?;
                    let (vendor_id, product_id) = get_printer_details();
                    let driver = rongta::SupportedDriver::Usb(vendor_id, product_id);

                    match msg.topic.as_str() {
                        "command/konan_pi/outline" => {
                            let params: OutlineTemplate = serde_json::from_str(&payload).unwrap();
                            let mut template = BoxTemplateBuilder::new(builder, pattern);
                            template
                                .set_lined(params.lined.unwrap_or_default())
                                .set_banner(params.banner);
                            if let Some(d) = params.date {
                                template.set_date_banner(d.into());
                            }
                            if let Some(rows) = params.rows {
                                template.set_rows(rows);
                            }
                            template.print(driver)?;
                        }
                        "command/konan_pi/habits" => {
                            let params: HabitTrackerTemplate =
                                serde_json::from_str(&payload).unwrap();
                            let mut template = HabitTrackerTemplateBuilder::new(
                                builder,
                                pattern,
                                params.habit,
                                params.start_date,
                                params.end_date,
                            );
                            template.print(driver)?;
                        }
                        "command/konan_pi/message" => {
                            let template = TipTapInterpreter::new(builder);
                            let params: PrintableMessage = serde_json::from_str(&payload).unwrap();
                            template.print(params.content, params.rows, driver)?;
                        }
                        _ => {
                            log::error!("Unsupported message topic")
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// Expand leading '~' to the user's home directory so paths like
// '~/.iot-device/certs/*' resolve correctly.
fn expand_home(p: &str) -> PathBuf {
    let home = || {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
    };

    if p == "~" {
        return home().unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return home()
            .map(|h| h.join(rest))
            .unwrap_or_else(|| PathBuf::from(p));
    }
    PathBuf::from(p)
}

fn configure_tls(cert_path: &str, key_path: &str, ca_path: &str) -> Result<TlsConfiguration> {
    let cert_path = expand_home(cert_path);
    let key_path = expand_home(key_path);
    let ca_path = expand_home(ca_path);

    // Load device/client certificate chain (DER)
    let mut cert_reader = BufReader::new(File::open(&cert_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "failed to open client certificate at '{}': {}",
                cert_path.display(),
                e
            ),
        )
    })?);
    let client_certs: Vec<CertificateDer<'static>> = certs(&mut cert_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse client certificate PEM(s) in '{}': {}",
                    cert_path.display(),
                    e
                ),
            )
        })?;
    if client_certs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "no client certificates found in '{}'. Ensure the file contains PEM-encoded certificate(s)",
                cert_path.display()
            ),
        )
        .into());
    }

    // Load private key (support PKCS#8 and PKCS#1/RSA)
    let mut key_reader = BufReader::new(File::open(&key_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "failed to open private key at '{}': {}",
                key_path.display(),
                e
            ),
        )
    })?);
    // Try PKCS#8 keys first
    let pkcs8_keys: Vec<PrivatePkcs8KeyDer<'static>> = pkcs8_private_keys(&mut key_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse PKCS#8 private key in '{}': {}",
                    key_path.display(),
                    e
                ),
            )
        })?;
    let mut keys: Vec<PrivateKeyDer<'static>> = pkcs8_keys.into_iter().map(Into::into).collect();

    if keys.is_empty() {
        // Retry as RSA (PKCS#1) if PKCS#8 not found
        let mut key_reader = BufReader::new(File::open(&key_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "failed to open private key at '{}': {}",
                    key_path.display(),
                    e
                ),
            )
        })?);
        let pkcs1_keys: Vec<PrivatePkcs1KeyDer<'static>> = rsa_private_keys(&mut key_reader)
            .collect::<Result<_, _>>()
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "failed to parse RSA (PKCS#1) private key in '{}': {}",
                        key_path.display(),
                        e
                    ),
                )
            })?;
        keys = pkcs1_keys.into_iter().map(Into::into).collect();
    }
    let private_key: PrivateKeyDer<'static> = keys.into_iter().next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "no usable private key found in '{}'. Ensure it is an unencrypted PKCS#8 or PKCS#1 (RSA) key",
                key_path.display()
            ),
        )
    })?;

    // Load Amazon Root CA(s) and build trust store
    let mut ca_reader = BufReader::new(File::open(&ca_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to open CA bundle at '{}': {}", ca_path.display(), e),
        )
    })?);
    let ca_der: Vec<CertificateDer<'static>> = certs(&mut ca_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse CA certificate PEM(s) in '{}': {}",
                    ca_path.display(),
                    e
                ),
            )
        })?;
    let mut root_cert_store = rustls::RootCertStore::empty();
    // Accept all parsable certs from the provided bundle
    let added = root_cert_store.add_parsable_certificates(ca_der);
    if added.0 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "no valid CA certificates found in '{}'. Verify the file contains PEM-encoded CA cert(s)",
                ca_path.display()
            ),
        )
        .into());
    }

    // Build rustls client config suitable for AWS IoT Core on port 8883
    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, private_key)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to build TLS client config (check certificate/key pair): {}",
                    e
                ),
            )
        })?;

    Ok(TlsConfiguration::Rustls(Arc::new(client_config)))
}

fn get_printer_details() -> (u16, u16) {
    (0x0FE6, 0x811E)
}
