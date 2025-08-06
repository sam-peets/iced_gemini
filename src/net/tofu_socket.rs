use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, LazyLock},
};

use rustls::{RootCertStore, pki_types::ServerName};
use url::Url;

use crate::net::tofu_cert_verifier::TofuCertVerifier;

static ROOT_CERT_STORE: LazyLock<RootCertStore> =
    LazyLock::new(|| RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()));

#[derive(Debug)]
pub struct TofuSocket {
    client: rustls::ClientConnection,
    sock: TcpStream,
}

impl TofuSocket {
    pub fn new<U: TryInto<Url> + std::fmt::Debug + Clone>(
        host: U,
        verifier: TofuCertVerifier,
    ) -> anyhow::Result<Self>
    where
        <U as TryInto<Url>>::Error: std::error::Error + Send + Sync + 'static,
    {
        let host: Url = host.try_into()?;

        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(ROOT_CERT_STORE.clone())
            .with_no_client_auth();
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(verifier));

        let server_name: ServerName<'static> = host
            .host()
            .ok_or_else(|| anyhow::anyhow!("can't get host from uri: {host}"))?
            .to_string()
            .try_into()?;

        let client = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        let sock = TcpStream::connect((
            host.host().unwrap().to_string(),
            host.port().unwrap_or(1965),
        ))
        .unwrap();

        Ok(TofuSocket { client, sock })
    }

    pub fn request(&mut self, request: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut stream = rustls::Stream::new(&mut self.client, &mut self.sock);

        stream.write_all(request)?;
        stream.flush()?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;

        Ok(buf)
    }
}
