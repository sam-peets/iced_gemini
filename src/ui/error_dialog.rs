use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill, Shrink},
    Theme,
    widget::{Column, Container, Row, button, container, text},
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
                .push(container(button("x").on_press(self.on_ok)).align_right(Fill)),
        )
        .padding(10)
        .height(Shrink)
        .align_y(Center)
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
