#[warn(clippy::all, clippy::pedantic)]
mod gemini;
mod net;
mod ui;

use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::{Column, Row, button, column, container, scrollable, text, text_input};
use iced::{Element, Font, Task, application};
use url::Url;

use crate::gemini::client::Client;
use crate::gemini::gemtext::Document;
use crate::gemini::response::Response;
use crate::ui::error_dialog::ErrorDialog;
use crate::ui::gemini_text::GeminiText;
use crate::ui::input_modal::{self, InputModal, InputRequest};
use crate::ui::modal::{self, Modal};

pub fn main() -> iced::Result {
    env_logger::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    let app = application(
        || {
            let t = Task::done(Message::PageLoad(
                url::Url::parse("gemini://geminiprotocol.net/").expect("Should never fail"),
            ));
            (GeminiClient::default(), t)
        },
        GeminiClient::update,
        GeminiClient::view,
    )
    .default_font(Font::with_name("Arial"));
    app.run()
}

struct GeminiClient {
    uri: String,
    document: Option<Document>,
    client: Client,
    scroll_id: scrollable::Id,
    scroll_position: AbsoluteOffset,
    history_back: Vec<(Document, AbsoluteOffset)>,
    history_forward: Vec<(Document, AbsoluteOffset)>,
    errors: Vec<String>,
    input_text: String,
    input_request: Option<InputRequest>,
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self {
            uri: Default::default(),
            document: Default::default(),
            client: Default::default(),
            scroll_id: scrollable::Id::unique(),
            history_back: Default::default(),
            history_forward: Default::default(),
            scroll_position: Default::default(),
            errors: Default::default(),
            input_text: Default::default(),
            input_request: Default::default(),
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
    BackButtonPressed,
    ForwardButtonPressed,
    Error(String),
    Scrolled(AbsoluteOffset),
    HomeButtonPressed,
    HideErrorModal(usize),
    OnPressError(String),
    OnSubmitInput,
    OnChangeInput(String),
    InputExpected(Url, Response),
}

impl GeminiClient {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UriChanged(uri) => {
                self.uri = uri;
            }
            Message::PageLoad(url) => {
                log::info!("PageLoad: opening url: {:?}", url);
                if url.scheme() != "gemini" {
                    if let Err(e) = opener::open(url.to_string()) {
                        return Task::done(Message::Error(e.to_string()));
                    }
                    return Task::none();
                }
                if let Some(doc) = self.document.clone() {
                    log::info!("PageLoad: adding {:?} to history", doc.url);
                    self.history_back.push((doc, self.scroll_position));
                    self.history_forward.clear();
                }
                let load_task = {
                    let url = url.clone();
                    let client = self.client;
                    Task::future(async move { client.load_page(&url) })
                };
                let scroll_task = scrollable::scroll_to(
                    self.scroll_id.clone(),
                    AbsoluteOffset { x: 0.0, y: 0.0 },
                );
                return Task::batch([load_task, scroll_task]);
            }
            Message::ButtonPressed(page) => {
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
                self.errors.push(e);
            }
            Message::BackButtonPressed => {
                log::info!(
                    "BackButtonPressed: history_back: {:?}, history_forward: {:?}",
                    self.history_back,
                    self.history_forward
                );
                if let Some((prev_doc, pos)) = self.history_back.pop() {
                    if let Some(current_doc) = self.document.clone() {
                        self.history_forward
                            .push((current_doc, self.scroll_position));
                    };
                    let scroll_task = scrollable::scroll_to(self.scroll_id.clone(), pos);
                    let load_task =
                        Task::done(Message::Loaded(prev_doc.url.clone(), Some(prev_doc)));
                    return Task::batch([scroll_task, load_task]);
                }
            }
            Message::ForwardButtonPressed => {
                log::info!(
                    "ForwardButtonPressed: history_back: {:?}, history_forward: {:?}",
                    self.history_back,
                    self.history_forward
                );
                if let Some((next_doc, pos)) = self.history_forward.pop() {
                    if let Some(current_doc) = self.document.clone() {
                        self.history_back.push((current_doc, self.scroll_position));
                    }
                    let scroll_task = scrollable::scroll_to(self.scroll_id.clone(), pos);
                    let load_task =
                        Task::done(Message::Loaded(next_doc.url.clone(), Some(next_doc)));
                    return Task::batch([scroll_task, load_task]);
                }
            }
            Message::Scrolled(absolute_offset) => {
                self.scroll_position = absolute_offset;
            }
            Message::HomeButtonPressed => {
                // TODO -> Custom home page
                return Task::done(Message::PageLoad(
                    url::Url::parse("gemini://geminiprotocol.net/").expect("Should never fail"),
                ));
            }
            Message::HideErrorModal(idx) => {
                self.errors.remove(idx);
            }
            Message::OnPressError(e) => {
                log::info!("FIXME: do something with the error")
            }
            Message::OnSubmitInput => {
                let t = if let Some(req) = &self.input_request {
                    let mut url = req.url.clone();
                    url.set_query(Some(&self.input_text));
                    Task::done(Message::PageLoad(url))
                } else {
                    Task::done(Message::Error(
                        "Tried to submit input while InputRequest was None".to_string(),
                    ))
                };
                self.input_request = None; // discard the request
                return t;
            }
            Message::OnChangeInput(s) => {
                self.input_text = s;
            }
            Message::InputExpected(url, res) => {
                log::info!("InputExpected: {url:?}, {res:?}");
                let prompt = res
                    .ctx
                    .unwrap_or("Input Expected (no information provided)".into());
                self.input_request = Some(InputRequest::new(url, prompt));
            }
        }
        Task::none()
    }

    fn url_bar(&self) -> Row<'_, Message> {
        Row::new()
            .push(button(GeminiText::new("⬅️").view()).on_press(Message::BackButtonPressed))
            .push(button(GeminiText::new("➡️").view()).on_press(Message::ForwardButtonPressed))
            .push(button(GeminiText::new("🏠").view()).on_press(Message::HomeButtonPressed))
            .push(
                text_input("uri", &self.uri)
                    .on_input(Message::UriChanged)
                    .on_submit(Message::GoButtonPressed),
            )
            .push(button("Go").on_press(Message::GoButtonPressed))
    }

    fn body(&self) -> Element<'_, Message> {
        if let Some(doc) = &self.document {
            scrollable(container(doc.view(|url| Message::ButtonPressed(url.clone()))).padding(20))
                .on_scroll(|v| Message::Scrolled(v.absolute_offset()))
                .id(self.scroll_id.clone())
                .width(iced::Fill)
                .height(iced::Fill)
                .into()
        } else {
            text("no page").into()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let base = column![self.url_bar(), self.body()].extend(self.errors.iter().enumerate().map(
            |(i, err)| {
                ErrorDialog::new(err.to_string(), Message::HideErrorModal(i))
                    .view(Message::OnPressError(err.clone()))
            },
        ));

        if let Some(input_request) = &self.input_request {
            let input_modal = input_request.modal();
            let modal = Modal::new(
                base.into(),
                input_modal
                    .view(
                        &self.input_text,
                        Message::OnChangeInput,
                        Message::OnSubmitInput,
                    )
                    .into(),
            );
            modal.view()
        } else {
            base.into()
        }
    }
}
