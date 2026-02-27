use crate::shared::driver;
use anyhow::{Context, bail};
use blueprint::{
    interpreter::tiptap::TipTapInterpreter,
    template::{
        box_outline::BoxTemplateBuilder, get_random_box_pattern,
        habit_tracker::HabitTrackerTemplateBuilder,
    },
};
use chrono::{DateTime, Utc};
use rongta::RongtaPrinter;
use rumqttc::{AsyncClient, ConnectionError, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};
use tiptap::JSONContent;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct KonanIotConfig {
    pub endpoint: String,
    pub port: u16,
    pub client_id: String,
    pub cert_path: PathBuf,
    pub private_key_path: PathBuf,
    pub root_trust_path: PathBuf,
}

impl KonanIotConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let endpoint = std::env::var("KONAN_IOT_ENDPOINT_URL")
            .with_context(|| "Missing KONAN_IOT_ENDPOINT_URL")?;

        let port = std::env::var("KONAN_IOT_PORT")
            .with_context(|| "Missing KONAN_IOT_PORT")?
            .parse::<u16>()
            .with_context(|| "KONAN_IOT_PORT not valid port number")?;

        let client_id =
            std::env::var("KONAN_IOT_CLIENT_ID").with_context(|| "Missing KONAN_IOT_CLIENT_ID")?;

        let cert_path = std::env::var("KONAN_CERTIFICATION_PATH")
            .with_context(|| "Missing KONAN_CERTIFICATION_PATH")?
            .into();

        let private_key_path = std::env::var("KONAN_PRIVATE_KEY_PATH")
            .with_context(|| "Missing KONAN_PRIVATE_KEY_PATH")?
            .into();

        let root_trust_path = std::env::var("KONAN_ROOT_OF_TRUST_PATH")
            .with_context(|| "Missing KONAN_ROOT_OF_TRUST_PATH")?
            .into();

        Ok(Self {
            endpoint,
            port,
            client_id,
            cert_path,
            private_key_path,
            root_trust_path,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct OutlineTemplate {
    rows: Option<u32>,
    date: Option<DateTime<Utc>>,
    banner: Option<String>,
    lined: Option<bool>,
}
#[derive(Debug, Deserialize, Serialize)]
struct PrintableMessage {
    content: JSONContent,
    rows: Option<u32>,
}
#[derive(Debug, Deserialize, Serialize)]
struct HabitTrackerTemplate {
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

enum MqttTopic {
    Habits,
    Message,
    Outline,
}
impl MqttTopic {
    fn as_topic(&self) -> &'static str {
        match self {
            MqttTopic::Habits => "command/konan_pi/habits",
            MqttTopic::Message => "command/konan_pi/message",
            MqttTopic::Outline => "command/konan_pi/outline",
        }
    }
    async fn subscribe_client(&self, client: &AsyncClient) -> anyhow::Result<()> {
        let topic = self.as_topic();
        client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to subscribe topic {}: {}", topic, e))?;
        Ok(())
    }
}
impl TryFrom<String> for MqttTopic {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "command/konan_pi/habits" => Ok(MqttTopic::Habits),
            "command/konan_pi/message" => Ok(MqttTopic::Message),
            "command/konan_pi/outline" => Ok(MqttTopic::Outline),
            _ => Err(anyhow::anyhow!("Unsupported variation")),
        }
    }
}

pub async fn handle_connect_command() -> anyhow::Result<()> {
    let config = KonanIotConfig::from_env()?;
    let mut mqttoptions = MqttOptions::new(config.client_id, config.endpoint, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    let tls_config = configure_tls(
        // "~/.iot-device/certs/konan.pem",
        config.cert_path,
        // "~/.iot-device/certs/konan_private.key",
        config.private_key_path,
        // "~/.iot-device/certs/AmazonRootCA1.pem",
        config.root_trust_path,
    )?;

    mqttoptions.set_transport(Transport::Tls(tls_config));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    MqttTopic::Habits.subscribe_client(&client).await?;
    MqttTopic::Message.subscribe_client(&client).await?;
    MqttTopic::Outline.subscribe_client(&client).await?;

    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                    if let Ok(topic) = MqttTopic::try_from(msg.topic) {
                        let builder = RongtaPrinter::new(true);
                        let pattern = get_random_box_pattern()?;

                        match topic {
                            MqttTopic::Habits => {
                                let params: HabitTrackerTemplate =
                                    serde_json::from_slice(&msg.payload).unwrap();
                                let mut template = HabitTrackerTemplateBuilder::new(
                                    builder,
                                    pattern,
                                    params.habit,
                                    params.start_date,
                                    params.end_date,
                                );
                                tokio::task::spawn_blocking(move || template.print(driver()));
                            }
                            MqttTopic::Message => {
                                let mut template = TipTapInterpreter::new(builder);
                                let params: PrintableMessage =
                                    serde_json::from_slice(&msg.payload).unwrap();
                                tokio::task::spawn_blocking(move || {
                                    template.print(params.content, params.rows, driver())
                                });
                            }
                            MqttTopic::Outline => {
                                let params: OutlineTemplate =
                                    serde_json::from_slice(&msg.payload).unwrap();
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
                                tokio::task::spawn_blocking(move || template.print(driver()));
                            }
                        }
                    } else {
                        log::warn!("Called invalid topic")
                    }
                }
            }
            Err(e) => {
                if is_fatal_error(&e) {
                    bail!("Fatal error: {}", e)
                } else {
                    log::error!("Non fatal error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

fn load_client_certs(path: &Path) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(File::open(path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to open client certificate at '{}': {}",
            path.display(),
            e
        )
    })?);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse client certificate PEM(s) in '{}': {}",
                path.display(),
                e
            )
        })?;
    if certs.is_empty() {
        bail!(
            "No client certificates found in '{}'. Ensure the file contains PEM-encoded certificate(s)",
            path.display()
        );
    }
    Ok(certs)
}

fn load_private_key(path: &Path) -> anyhow::Result<PrivateKeyDer<'static>> {
    // Attempt PKCS#8 first (covers both RSA and EC keys in the modern format).
    let mut reader = BufReader::new(File::open(path).map_err(|e| {
        anyhow::anyhow!("Failed to open private key at '{}': {}", path.display(), e)
    })?);
    let pkcs8: Vec<PrivatePkcs8KeyDer<'static>> = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse PKCS#8 private key in '{}': {}",
                path.display(),
                e
            )
        })?;
    if let Some(key) = pkcs8.into_iter().next() {
        return Ok(key.into());
    }

    // Fall back to legacy PKCS#1 RSA keys (PEM label "RSA PRIVATE KEY").
    // The reader must be re-opened because rustls_pemfile consumes it entirely,
    // even when no matching blocks are found.
    let mut reader = BufReader::new(File::open(path).map_err(|e| {
        anyhow::anyhow!("Failed to open private key at '{}': {}", path.display(), e)
    })?);
    let pkcs1: Vec<PrivatePkcs1KeyDer<'static>> = rustls_pemfile::rsa_private_keys(&mut reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse RSA (PKCS#1) private key in '{}': {}",
                path.display(),
                e
            )
        })?;
    if let Some(key) = pkcs1.into_iter().next() {
        return Ok(key.into());
    }

    // Fall back to legacy bare EC keys (PEM label "EC PRIVATE KEY").
    // Re-open for the same reason as above.
    let mut reader = BufReader::new(File::open(path).map_err(|e| {
        anyhow::anyhow!("Failed to open private key at '{}': {}", path.display(), e)
    })?);
    let ec: Vec<PrivateKeyDer<'static>> = rustls_pemfile::ec_private_keys(&mut reader)
        .map(|r| r.map(Into::into))
        .collect::<Result<_, _>>()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse EC private key in '{}': {}",
                path.display(),
                e
            )
        })?;
    ec.into_iter().next().ok_or_else(|| {
        anyhow::anyhow!(
            "No usable private key found in '{}'. Ensure the file contains an unencrypted \
             PKCS#8 (RSA or EC), PKCS#1 (RSA), or SEC1 (EC) PEM-encoded private key.",
            path.display()
        )
    })
}

fn load_root_cert_store(path: &Path) -> anyhow::Result<rustls::RootCertStore> {
    let mut reader =
        BufReader::new(File::open(path).map_err(|e| {
            anyhow::anyhow!("Failed to open CA bundle at '{}': {}", path.display(), e)
        })?);
    let ca_der: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<_, _>>()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse CA certificate PEM(s) in '{}': {}",
                path.display(),
                e
            )
        })?;
    let mut store = rustls::RootCertStore::empty();
    let (added, ignored) = store.add_parsable_certificates(ca_der);
    if ignored > 0 {
        bail!(
            "{} CA certificate(s) failed to parse in '{}' and were ignored. \
             Verify all entries in the file are valid PEM-encoded CA certificates.",
            ignored,
            path.display()
        );
    }
    if added == 0 {
        bail!(
            "No valid CA certificates found in '{}'. Verify the file contains PEM-encoded CA cert(s)",
            path.display()
        );
    }
    Ok(store)
}

fn configure_tls(
    cert_path: PathBuf,
    key_path: PathBuf,
    ca_path: PathBuf,
) -> anyhow::Result<TlsConfiguration> {
    let client_certs = load_client_certs(cert_path.as_path())?;
    let private_key = load_private_key(key_path.as_path())?;
    let root_store = load_root_cert_store(ca_path.as_path())?;
    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_certs, private_key)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to build TLS client config. Ensure '{}' and '{}' are a matching \
                 certificate/key pair: {}",
                cert_path.display(),
                key_path.display(),
                e
            )
        })?;
    Ok(TlsConfiguration::Rustls(Arc::new(client_config)))
}

fn is_fatal_error(error: &ConnectionError) -> bool {
    match error {
        // The only genuinely transient cases — network blips that resolve on their own
        ConnectionError::NetworkTimeout => false,
        ConnectionError::FlushTimeout => false,
        ConnectionError::Io(e) => !matches!(
            e.kind(),
            std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::UnexpectedEof
        ),

        // Everything else is fatal
        ConnectionError::Tls(_) => true,
        ConnectionError::ConnectionRefused(_) => true,
        ConnectionError::NotConnAck(_) => true,
        ConnectionError::RequestsDone => true,
        ConnectionError::MqttState(_) => true,
    }
}
