use std::{fs, path::Path};

use rcgen::{CertificateParams, DistinguishedName, KeyPair};

use rustls::{
    DigitallySignedStruct, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{ring, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct TlsIdentity {
    pub cert: String,
    pub key: String,
}
pub type Fingerprint = [u8; 32];

#[derive(Debug)]
pub struct TofuVerifier {
    // ip:port -> fingerprint
    known: Arc<RwLock<HashMap<String, Fingerprint>>>,
}

impl TofuVerifier {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            known: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

const CERT_PATH: &str = "cert.pem";
const KEY_PATH: &str = "key.pem";

pub fn load_or_generate(device_name: &str) -> TlsIdentity {
    if Path::new(CERT_PATH).exists() && Path::new(KEY_PATH).exists() {
        tracing::info!("Loading existing TLS certificate");
        return TlsIdentity {
            cert: fs::read_to_string(CERT_PATH).expect("Failed to read cert"),
            key: fs::read_to_string(KEY_PATH).expect("Failed to read key"),
        };
    };

    tracing::info!("Generating new TLS cert for {}", device_name);

    let key_pair = KeyPair::generate().expect("Failed to generate keypair");

    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, device_name);

    let cert = params.self_signed(&key_pair).expect("Failed to self-sign");

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    fs::write(CERT_PATH, &cert_pem).expect("Failed to write cert");
    fs::write(KEY_PATH, &key_pem).expect("Failed to write key");

    return TlsIdentity {
        cert: cert_pem,
        key: key_pem,
    };
}

pub fn build_http_client(verifier: Arc<TofuVerifier>) -> reqwest::Client {
    let rustls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    reqwest::Client::builder()
        .use_preconfigured_tls(rustls_config)
        .build()
        .expect("Failed to build HTTP client")
}

impl ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let fingerprint: Fingerprint = Sha256::digest(end_entity.as_ref()).into();
        let key = server_name.to_str().to_string();

        let mut known = self.known.write().unwrap();
        match known.get(&key) {
            Some(existing) if *existing != fingerprint => {
                tracing::warn!(peer = %key, "TLS certificate mismatch, possible MITM!");
                return Err(rustls::Error::General("Certificate mismatch".into()));
            }
            None => {
                tracing::info!(peer = %key, "Trusting new peer certificate");
                known.insert(key, fingerprint);
            }
            _ => {} // known and fingerprint matches
        }

        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
