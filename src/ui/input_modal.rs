use iced::{
    Element,
    Length::{Fill, Shrink},
    widget::{Column, button, container, text_input},
};
use url::Url;

use crate::ui::gemini_text::GeminiText;

#[derive(Debug, Clone)]
pub struct InputRequest {
    pub url: Url,
    pub prompt: String,
}

impl InputRequest {
    pub fn new(url: Url, prompt: String) -> Self {
        Self { url, prompt }
    }

    pub fn modal(&self) -> InputModal {
        InputModal {
            prompt: self.prompt.clone(),
        }
    }
}

pub struct InputModal {
    prompt: String,
}

impl InputModal {
    pub fn view<'a, Message: Clone + 'a>(
        self,
        value: &str,
        on_change: fn(String) -> Message,
        on_submit: Message,
    ) -> Element<'a, Message> {
        let text_row = container(GeminiText::new(&self.prompt).view())
            .center_x(Fill)
            .center_y(Shrink)
            .padding(10);

        let input_row = container(
            text_input("input", value)
                .on_input(on_change)
                .on_submit(on_submit.clone()),
        )
        .center_x(Fill)
        .center_y(Shrink)
        .padding(10);

        let button_row = container(button("Submit").on_press(on_submit.clone()))
            .center_x(Fill)
            .center_y(Shrink)
            .padding(10);
        Column::new()
            .push(text_row)
            .push(input_row)
            .push(button_row)
            .into()
    }
}
