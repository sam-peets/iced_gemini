mod gemini;
mod net;
mod ui;

use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::{Column, Row, button, column, container, scrollable, text, text_input};
use iced::{Element, Task, application};
use url::Url;

use crate::gemini::client::Client;
use crate::gemini::gemtext::{Document, Line};

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
    scroll_id: scrollable::Id,
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self {
            uri: Default::default(),
            document: Default::default(),
            client: Default::default(),
            scroll_id: scrollable::Id::unique(),
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
    Error(String),
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
                    if let Err(e) = opener::open(url.to_string()) {
                        return Task::done(Message::Error(e.to_string()));
                    }
                    return Task::none();
                }

                let load_task = {
                    let url = url.clone();
                    let client = self.client;
                    Task::future(async move { client.load_page(&url).await })
                };
                let scroll_task = scrollable::scroll_to(
                    self.scroll_id.clone(),
                    AbsoluteOffset { x: 0.0, y: 0.0 },
                );
                return Task::batch([load_task, scroll_task]);
            }
            Message::ButtonPressed(page) => {
                // TODO: add to history, etc.
                return Task::done(Message::PageLoad(page));
            }
            Message::GoButtonPressed => {
                return match Url::parse(&self.uri) {
                    Ok(url) => Task::done(Message::PageLoad(url)),
                    Err(e) => Task::done(Message::Error(e.to_string())),
                };
            }
            Message::Loaded(url, document) => {
                self.uri = url.to_string();
                self.document = document;
            }
            Message::Error(e) => {
                // TODO - client error handling, maybe a modal?
                log::error!("Error: {e:?}");
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
            .id(self.scroll_id.clone())
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
