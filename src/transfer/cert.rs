use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use std::{fs, path::Path};

use crate::transfer::tls::{TlsError, TlsIdentity};

pub fn load_or_generate(
    device_name: &str,
    cert_path: &str,
    key_path: &str,
) -> Result<TlsIdentity, TlsError> {
    if Path::new(cert_path).exists() && Path::new(key_path).exists() {
        return load_certificate(cert_path, key_path);
    }
    generate_certificate(cert_path, key_path, device_name)
}

fn load_certificate(cert_path: &str, key_path: &str) -> Result<TlsIdentity, TlsError> {
    tracing::info!("loading existing TLS certificate");
    let cert = fs::read_to_string(cert_path).map_err(TlsError::FailedIO)?;
    let key = fs::read_to_string(key_path).map_err(TlsError::FailedIO)?;

    Ok(TlsIdentity { cert, key })
}

fn generate_certificate(
    cert_path: &str,
    key_path: &str,
    device_name: &str,
) -> Result<TlsIdentity, TlsError> {
    tracing::info!("Generating new TLS cert for {}", device_name);

    let key_pair = KeyPair::generate().map_err(TlsError::FailedCertGen)?;

    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, device_name);

    let cert = params
        .self_signed(&key_pair)
        .map_err(TlsError::FailedCertGen)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    fs::write(cert_path, &cert_pem).map_err(TlsError::FailedIO)?;
    fs::write(key_path, &key_pem).map_err(TlsError::FailedIO)?;

    Ok(TlsIdentity {
        cert: cert_pem,
        key: key_pem,
    })
}
