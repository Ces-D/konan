use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // AWS IoT Core endpoint (replace with your endpoint)
    let endpoint = "your-endpoint.iot.us-east-1.amazonaws.com";
    let port = 8883;
    let client_id = "raspberry-pi-5-subscriber";

    // Set up MQTT options
    let mut mqttoptions = MqttOptions::new(client_id, endpoint, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    // Configure TLS
    let tls_config = configure_tls(
        ".iot-device/certs/konan.pem",
        ".iot-device/certs/konan_private.key",
        ".iot-device/certs/AmazonRootCA1.pem",
    )?;

    mqttoptions.set_transport(Transport::Tls(tls_config));

    // Create client and event loop
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to a topic
    client
        .subscribe("command/konan_pi/habits", QoS::AtLeastOnce)
        .await?;
    client
        .subscribe("command/konan_pi/message", QoS::AtLeastOnce)
        .await?;
    client
        .subscribe("command/konan_pi/outline", QoS::AtLeastOnce)
        .await?;

    // Handle incoming messages
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                    let payload = String::from_utf8_lossy(&msg.payload);
                    println!("Received message on topic '{}': {}", msg.topic, payload);
                }
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn configure_tls(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
) -> Result<TlsConfiguration, Box<dyn Error>> {
    // Load device/client certificate chain (DER)
    let mut cert_reader = BufReader::new(File::open(cert_path)?);
    let client_certs: Vec<CertificateDer<'static>> =
        certs(&mut cert_reader).collect::<Result<_, _>>()?;

    // Load private key (support PKCS#8 and PKCS#1/RSA)
    let mut key_reader = BufReader::new(File::open(key_path)?);
    // Try PKCS#8 keys first
    let pkcs8_keys: Vec<PrivatePkcs8KeyDer<'static>> =
        pkcs8_private_keys(&mut key_reader).collect::<Result<_, _>>()?;
    let mut keys: Vec<PrivateKeyDer<'static>> = pkcs8_keys.into_iter().map(Into::into).collect();

    if keys.is_empty() {
        // Retry as RSA (PKCS#1) if PKCS#8 not found
        let mut key_reader = BufReader::new(File::open(key_path)?);
        let pkcs1_keys: Vec<PrivatePkcs1KeyDer<'static>> =
            rsa_private_keys(&mut key_reader).collect::<Result<_, _>>()?;
        keys = pkcs1_keys.into_iter().map(Into::into).collect();
    }
    let private_key: PrivateKeyDer<'static> = keys
        .into_iter()
        .next()
        .ok_or("no private key found in key file")?;

    // Load Amazon Root CA(s) and build trust store
    let mut ca_reader = BufReader::new(File::open(ca_path)?);
    let ca_der: Vec<CertificateDer<'static>> = certs(&mut ca_reader).collect::<Result<_, _>>()?;
    let mut root_cert_store = rustls::RootCertStore::empty();
    // Accept all parsable certs from the provided bundle
    root_cert_store.add_parsable_certificates(ca_der);

    // Build rustls client config suitable for AWS IoT Core on port 8883
    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, private_key)?;

    Ok(TlsConfiguration::Rustls(Arc::new(client_config)))
}
