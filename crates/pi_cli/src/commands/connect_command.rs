use crate::{config::KonanIotConfig, print_ops::enqueue_print};
use anyhow::bail;
use chrono::{DateTime, Local, NaiveTime, Utc};
use cli_shared::PrintTask;
use rumqttc::{AsyncClient, ConnectionError, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
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
    content: String,
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

const ACTIVE_WINDOW_START: u32 = 9;
const ACTIVE_WINDOW_END: u32 = 22;

fn is_within_active_window() -> bool {
    let now = Local::now().time();
    let start = NaiveTime::from_hms_opt(ACTIVE_WINDOW_START, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(ACTIVE_WINDOW_END, 0, 0).unwrap();
    now >= start && now < end
}

fn duration_until_window_start() -> Duration {
    let now = Local::now();
    let today_start = now
        .date_naive()
        .and_hms_opt(ACTIVE_WINDOW_START, 0, 0)
        .unwrap();
    let target = if now.time() >= NaiveTime::from_hms_opt(ACTIVE_WINDOW_START, 0, 0).unwrap() {
        // Already past today's start, aim for tomorrow
        today_start + chrono::Duration::days(1)
    } else {
        today_start
    };
    let diff = target - now.naive_local();
    diff.to_std().unwrap_or(Duration::from_secs(60))
}

pub async fn handle_connect_command(config: KonanIotConfig) -> anyhow::Result<()> {
    let tls_config = configure_tls(
        config.cert_path,
        config.private_key_path,
        config.root_trust_path,
    )?;

    loop {
        if !is_within_active_window() {
            let wait = duration_until_window_start();
            log::info!(
                "Outside active window ({:02}:00–{:02}:00). Sleeping for {}m until next window.",
                ACTIVE_WINDOW_START,
                ACTIVE_WINDOW_END,
                wait.as_secs() / 60
            );
            tokio::time::sleep(wait).await;
            continue;
        }

        log::info!("Within active window. Connecting to MQTT broker.");

        let mut mqttoptions = MqttOptions::new(&config.client_id, &config.endpoint, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        mqttoptions.set_transport(Transport::Tls(tls_config.clone()));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        MqttTopic::Habits.subscribe_client(&client).await?;
        MqttTopic::Message.subscribe_client(&client).await?;
        MqttTopic::Outline.subscribe_client(&client).await?;

        loop {
            if !is_within_active_window() {
                log::info!("Active window ended. Disconnecting from MQTT broker.");
                let _ = client.disconnect().await;
                break;
            }

            match eventloop.poll().await {
                Ok(notification) => {
                    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                        if let Ok(topic) = MqttTopic::try_from(msg.topic) {
                            match topic {
                                MqttTopic::Habits => {
                                    let params: HabitTrackerTemplate =
                                        serde_json::from_slice(&msg.payload).unwrap();
                                    enqueue_print(PrintTask::HabitTracker {
                                        cut: true,
                                        habit: params.habit,
                                        start_date: params.start_date,
                                        end_date: params.end_date,
                                    })
                                    .await;
                                }
                                MqttTopic::Message => {
                                    let params: PrintableMessage =
                                        serde_json::from_slice(&msg.payload).unwrap();
                                    enqueue_print(PrintTask::Markdown {
                                        cut: true,
                                        content: params.content,
                                        rows: params.rows,
                                    })
                                    .await;
                                }
                                MqttTopic::Outline => {
                                    let params: OutlineTemplate =
                                        serde_json::from_slice(&msg.payload).unwrap();
                                    let date = params.date;
                                    enqueue_print(PrintTask::BoxTemplate {
                                        cut: true,
                                        rows: params.rows,
                                        lined: params.lined.unwrap_or_default(),
                                        banner: params.banner,
                                        date,
                                    })
                                    .await;
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
