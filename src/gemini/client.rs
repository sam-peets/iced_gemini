use std::str::FromStr;

use rustls::crypto::CryptoProvider;
use thiserror::Error;
use url::Url;

use crate::{
    gemini::response::{self, Response},
    net::{tofu_cert_verifier::TofuCertVerifier, tofu_socket::TofuSocket},
};

#[derive(Debug, Clone, Copy)]
pub struct Client {
    verifier: TofuCertVerifier,
}

#[derive(Error, Debug)]
enum ClientError {
    #[error("ClientError: Missing context: {0}")]
    MissingContext(String),
}
impl Client {
    pub fn new() -> Self {
        Self {
            verifier: TofuCertVerifier::new(
                CryptoProvider::get_default()
                    .unwrap()
                    .signature_verification_algorithms,
            ),
        }
    }

    pub async fn request(&self, url: &Url) -> anyhow::Result<(Url, Response)> {
        let mut sock = TofuSocket::new(url.clone(), self.verifier)?;

        let res = sock.request(format!("{url}\r\n").as_bytes())?;
        let mut r = response::Response::from_str(str::from_utf8(&res)?)?;

        while (30..=39).any(|x| x == r.status as u8) {
            // redirect
            log::info!("Client: request: redirecting to {:?}", r.ctx);
            let ctx = r.ctx.ok_or(ClientError::MissingContext(format!(
                "No redirect url on status {:?}",
                r.status
            )))?;
            let redirect = url.join(&ctx)?;
            let res = sock.request(format!("{redirect}\r\n").as_bytes())?;
            r = response::Response::from_str(str::from_utf8(&res)?)?;
        }
        if let 30..=39 = r.status as u8 {};

        Ok((url.clone(), r))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
