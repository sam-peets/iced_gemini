use iced::{
    Element,
    Length::Fill,
    Theme,
    widget::{Column, Row, button, container, text},
};

#[derive(Clone)]
pub struct ErrorDialog<Message: Clone> {
    text: String,
    on_ok: Message,
}

impl<Message: Clone> ErrorDialog<Message> {
    pub fn new(text: String, on_ok: Message) -> Self {
        Self { text, on_ok }
    }

    pub fn from_error(err: impl std::error::Error, on_ok: Message) -> Self {
        Self::new(err.to_string(), on_ok)
    }

    pub fn view<'a>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        container(
            Row::new()
                .push(text(self.text))
                .push(button("x").on_press(self.on_ok))
                .width(Fill),
        )
        .style(|t: &Theme| {
            let pal = t.extended_palette();
            container::Style {
                background: Some(pal.danger.weak.color.into()),
                ..Default::default()
            }
        })
        .into()
    }
}
