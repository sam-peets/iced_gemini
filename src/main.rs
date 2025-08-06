pub mod gemini;
mod net;

use std::str::FromStr;

use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::{Column, Row, button, column, container, scrollable, text, text_input};
use iced::{Element, Font, Task, application};
use rustls::crypto::CryptoProvider;
use url::Url;

use crate::gemini::client::Client;
use crate::gemini::gemtext::{Document, Line};
use crate::gemini::response::{self, Response};
use crate::net::tofu_cert_verifier::TofuCertVerifier;
use crate::net::tofu_socket::TofuSocket;

pub fn main() -> iced::Result {
    env_logger::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    let app = application("iced out", GeminiClient::update, GeminiClient::view);
    // .default_font(Font::with_name("Times New Roman"));
    app.run()
}

struct GeminiClient {
    uri: String,
    document: Option<Document>,
    client: Client,
    scrollId: scrollable::Id,
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self {
            uri: Default::default(),
            document: Default::default(),
            client: Default::default(),
            scrollId: scrollable::Id::unique(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    UriChanged(String),
    PageLoad(Url),
    Loaded(Url, Option<Document>),
    ButtonPressed(Url), // current page, path
    GoButtonPressed,
}

impl GeminiClient {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UriChanged(uri) => {
                self.uri = uri;
            }
            Message::PageLoad(url) => {
                log::info!("GoButtonPressed: opening scheme: {:?}", url.scheme());
                if url.scheme() != "gemini" {
                    opener::open(url.to_string()).expect("Failed to open");
                    return Task::none();
                }
                let load_task = {
                    let url = url.clone();
                    let client = self.client;
                    Task::future(async move {
                        let (url, response) = client.request(&url).await.unwrap();
                        let document = Document::parse(&url, &response.body.unwrap()).ok();
                        Message::Loaded(url, document)
                    })
                };
                let scroll_task =
                    scrollable::scroll_to(self.scrollId.clone(), AbsoluteOffset { x: 0.0, y: 0.0 });
                return Task::batch([load_task, scroll_task]);
            }
            Message::ButtonPressed(page) => {
                // TODO: add to history, etc.
                return Task::done(Message::PageLoad(page));
            }
            Message::GoButtonPressed => {
                let url = Url::parse(&self.uri).unwrap();
                return Task::done(Message::PageLoad(url));
            }
            Message::Loaded(url, document) => {
                self.uri = url.to_string();
                self.document = document;
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
            .id(self.scrollId.clone())
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
        } else {
            text("no page").into()
        }
    }

    fn view(&self) -> Column<'_, Message> {
        column![self.url_bar(), self.body()]
    }
}
