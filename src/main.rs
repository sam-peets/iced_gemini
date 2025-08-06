pub mod gemini;
mod net;

use std::str::FromStr;

use iced::widget::{
    Column, Container, Row, Text, button, column, container, scrollable, text, text_input,
};
use iced::{Center, Element, Task};
use rustls::crypto::CryptoProvider;
use url::Url;

use crate::gemini::gemtext::{Document, Line};
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

fn load_page(url: &Url) -> anyhow::Result<Response> {
    log::info!("load_page: {url}");
    let mut sock = TofuSocket::new(
        url.clone(),
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
    document: Option<Document>,
}

#[derive(Debug, Clone)]
enum Message {
    UriChanged(String),
    LoadPage(Url),
    ButtonPressed(Url), // current page, path
    GoButtonPressed,
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UriChanged(uri) => {
                self.uri = uri;
            }
            Message::LoadPage(u) => {
                self.uri = u.to_string();
                let res = load_page(&u).unwrap();
                self.document = Document::parse(&u, &res.body.unwrap()).ok();
            }
            Message::ButtonPressed(page) => {
                // TODO: add to history, etc.
                return Task::done(Message::LoadPage(page));
            }
            Message::GoButtonPressed => {
                let mut url = Url::parse(&self.uri).unwrap();
                url.set_scheme("gemini").unwrap();
                return Task::done(Message::LoadPage(url));
            }
        }
        Task::none()
    }

    fn url_bar(&self) -> Row<'_, Message> {
        Row::new()
            .push(text_input("uri", &self.uri).on_input(Message::UriChanged))
            .push(button("Go").on_press(Message::GoButtonPressed))
    }

    fn body(&self) -> Element<'_, Message> {
        if let Some(doc) = &self.document {
            scrollable(
                container(Column::from_vec(
                    doc.lines
                        .iter()
                        .map(|l| {
                            l.view(|l| match l {
                                Line::Link(url, _) => Message::ButtonPressed(url.clone()),
                                _ => unreachable!(),
                            })
                        })
                        .collect(),
                ))
                .padding(20),
            )
            .into()
        } else {
            text("no page").into()
        }
    }

    fn view(&self) -> Column<'_, Message> {
        column![self.url_bar(), self.body()]
    }
}
