use iced::{
    Element,
    widget::{container, tooltip},
};

pub struct GeminiTooltip<'a, Message: Clone + 'a> {
    contents: Element<'a, Message>,
    tooltip: Element<'a, Message>,
}

impl<'a, Message: Clone> GeminiTooltip<'a, Message> {
    pub fn new(
        contents: impl Into<Element<'a, Message>>,
        tooltip: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            contents: contents.into(),
            tooltip: tooltip.into(),
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        tooltip(
            self.contents,
            container(self.tooltip).style(container::bordered_box),
            tooltip::Position::Right,
        )
        .into()
    }
}
