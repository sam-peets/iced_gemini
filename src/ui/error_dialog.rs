use iced::{
    Element,
    widget::{Column, button, center, column, container, text},
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
            Column::new()
                .push(text(self.text))
                .push(button("Ok").on_press(self.on_ok)),
        )
        .padding(20)
        .into()
    }
}
