use iced::{
    Border, Color, Element, Shadow,
    widget::{container, tooltip},
};

pub struct GeminiTooltip<'a, Message: Clone + 'a> {
    contents: Element<'a, Message>,
    tooltip: Element<'a, Message>,
}

impl<'a, Message: Clone> GeminiTooltip<'a, Message> {
    pub fn new(contents: Element<'a, Message>, tooltip: Element<'a, Message>) -> Self {
        Self { contents, tooltip }
    }

    pub fn view(self) -> Element<'a, Message> {
        tooltip(
            self.contents,
            container(self.tooltip).style(|theme| {
                let style = container::bordered_box(theme);
                style
            }),
            tooltip::Position::FollowCursor,
        )
        .into()
    }
}
