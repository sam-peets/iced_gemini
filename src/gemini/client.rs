use iced::widget::image::Handle;
use rustls::crypto::CryptoProvider;
use thiserror::Error;
use url::Url;

use crate::{
    Message,
    gemini::{
        Status,
        gemtext::{Document, Line},
        response::Response,
    },
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
                    .expect("TofuCertVerifier: no default CryptoProvider")
                    .signature_verification_algorithms,
            ),
        }
    }

    fn success(url: Url, response: Response) -> Message {
        log::info!("load_page: Success! Rendering page");
        let Some(body) = response.body else {
            return Message::Error("No response body".into());
        };

        log::info!("success: got MIME type: {:?}", response.ctx);
        let mime: mime::Mime = match response.ctx.unwrap_or("text/gemini".into()).parse() {
            Ok(x) => x,
            Err(e) => return Message::Error(e.to_string()),
        };

        match (mime.type_(), mime.subtype()) {
            (mime::TEXT, x) if x.as_str() == "gemini" => {
                let utf8_body = match String::from_utf8(body) {
                    Ok(x) => x,
                    Err(e) => {
                        return Message::Error(format!("body contains bad utf8 data: {e:?}"));
                    }
                };

                let document = match Document::parse(&url, &utf8_body) {
                    Ok(x) => x,
                    Err(e) => return Message::Error(e.to_string()),
                };

                Message::Loaded(url, Some(document))
            }
            (mime::IMAGE, _) => {
                let handle = Handle::from_bytes(body);
                let doc = Document {
                    url: url.clone(),
                    lines: vec![Line::Image(handle)],
                };
                Message::Loaded(url, Some(doc))
            }
            _ => Message::Error("unsupported MIME type".into()),
        }
    }

    pub fn load_page(&self, url: &Url) -> Message {
        let (url, response) = match self.request(url) {
            Ok(x) => x,
            Err(e) => return Message::Error("load_page: request: ".to_string() + &e.to_string()),
        };

        match response.status {
            Status::Success => Client::success(url, response),
            Status::InputExpected => Message::InputExpected(url, response),
            _ => Message::Error(format!("got bad status: {response:?}")),
        }
    }

    pub fn request(&self, url: &Url) -> anyhow::Result<(Url, Response)> {
        let mut sock = TofuSocket::new(url.clone(), self.verifier)?;

        let res = sock.request(format!("{url}\r\n").as_bytes())?;
        let mut r: Response = (&res[..]).try_into()?;
        let mut url = url.clone();

        while (30..=39).any(|x| x == r.status as u8) {
            // redirect
            log::info!("Client: request: redirecting to {:?}", r.ctx);
            let ctx = r.ctx.ok_or(ClientError::MissingContext(format!(
                "No redirect url on status {:?}",
                r.status
            )))?;

            // a socket is only good for one gemini request-response cycle, we need to make a new one
            url = url.join(&ctx)?;
            let mut sock = TofuSocket::new(url.clone(), self.verifier)?;
            let res = sock.request(format!("{url}\r\n").as_bytes())?;
            log::info!("Client: request: res: {:?}", str::from_utf8(&res));
            r = (&res[..]).try_into()?;
        }

        Ok((url, r))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
