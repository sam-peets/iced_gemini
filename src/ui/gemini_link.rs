use iced::{Color, Element, widget::button};
use url::Url;

use crate::ui::{gemini_text::GeminiText, gemini_tooltip::GeminiTooltip};

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

    pub fn view<'a>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let button_text = self.friendly.unwrap_or(self.url.to_string());

        let tooltip_host = if let Some(host_str) = self.url.host_str() {
            format!("{}://{}", self.url.scheme(), host_str)
        } else {
            // degenerate case where there isn't a host (can this ever happen?)
            // just use the entire url, better than nothing
            format!("{}", self.url)
        };

        let contents = button(GeminiText::new(&button_text).view())
            .style(|theme, status| {
                let style = button::primary(theme, status);
                button::Style {
                    background: None,
                    text_color: Color::from_rgb(0.2, 0.2, 0.8),
                    ..style
                }
            })
            .on_press((self.on_press)(&self.url));
        let tooltip = GeminiText::new(&tooltip_host).view();
        GeminiTooltip::new(contents, tooltip).view()
    }
}
