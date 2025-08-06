pub mod gemini;
mod net;

use std::str::FromStr;

use iced::widget::{
    Column, Container, Row, button, column, container, scrollable, text, text_input,
};
use iced::{Center, Element, Task};
use rustls::crypto::CryptoProvider;

use crate::gemini::gemtext::Document;
use crate::gemini::response::{self, Response};
use crate::net::tofu_cert_verifier::TofuCertVerifier;
use crate::net::tofu_socket::TofuSocket;

pub fn main() -> iced::Result {
    env_logger::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    iced::run("iced out gemini", Counter::update, Counter::view)
}

fn load_page(url: &str) -> anyhow::Result<Response> {
    log::info!("load_page: {url}");
    let mut sock = TofuSocket::new(
        url,
        TofuCertVerifier::new(
            CryptoProvider::get_default()
                .unwrap()
                .signature_verification_algorithms,
        ),
    )?;

    let res = sock.request(format!("{url}\r\n").as_bytes())?;

    Ok(response::Response::from_str(str::from_utf8(&res)?)?)
}

#[derive(Default)]
struct Counter {
    uri: String,
    document: Document,
}

#[derive(Debug, Clone)]
enum Message {
    UriChanged(String),
    LoadPage,
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UriChanged(uri) => {
                self.uri = uri;
            }
            Message::LoadPage => {
                let res = load_page(&self.uri).unwrap();
                log::info!("res: {res:?}");
                self.document = Document::parse(&res.body.unwrap()).unwrap();
            }
        }
        Task::none()
    }

    fn url_bar(&self) -> Row<'_, Message> {
        Row::new()
            .push(text_input("uri", &self.uri).on_input(Message::UriChanged))
            .push(button("Go").on_press(Message::LoadPage))
    }

    fn body(&self) -> Element<'_, Message> {
        scrollable(
            container(Column::from_vec(
                self.document.lines.iter().map(|l| l.view()).collect(),
            ))
            .padding(20),
        )
        .into()
    }

    fn view(&self) -> Column<'_, Message> {
        column![self.url_bar(), self.body()]
    }
}
