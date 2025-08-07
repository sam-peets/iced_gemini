use iced::{
    Color, Element,
    widget::{button, tooltip},
};
use url::Url;

use crate::ui::gemini_text::GeminiText;

pub struct GeminiLink<Message: Clone> {
    url: Url,
    friendly: Option<String>,
    on_press: fn(&Url) -> Message,
}

impl<Message: Clone> GeminiLink<Message> {
    pub fn new(url: Url, friendly: Option<String>, on_press: fn(&Url) -> Message) -> Self {
        Self {
            url,
            friendly,
            on_press,
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let button_text = self.friendly.clone().unwrap_or(self.url.to_string());
        let tooltip_host = self
            .url
            .host()
            .map(|x| x.to_string())
            .unwrap_or(self.url.to_string());
        let tooltip_text = format!("({tooltip_host})");

        tooltip(
            button(GeminiText::new(&button_text).view())
                .style(|theme, status| {
                    let style = button::primary(theme, status);
                    button::Style {
                        background: None,
                        text_color: Color::from_rgb(0.2, 0.2, 0.8),
                        ..style
                    }
                })
                .on_press((self.on_press)(&self.url)),
            GeminiText::new(&tooltip_text).view(),
            tooltip::Position::Right,
        )
        .into()
    }
}
