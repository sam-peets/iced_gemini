use iced::{
    Border, Color, Element,
    widget::{center, container, opaque, stack},
};

pub struct Modal<'a, Message: Clone + 'a> {
    base: Element<'a, Message>,
    contents: Element<'a, Message>,
}

impl<'a, Message: Clone + 'a> Modal<'a, Message> {
    pub fn new(
        base: impl Into<Element<'a, Message>>,
        contents: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            base: base.into(),
            contents: contents.into(),
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        let contents = opaque(
            center(
                container(self.contents)
                    .style(|t| {
                        let palette = t.extended_palette();
                        container::Style {
                            background: Some(palette.background.base.color.into()),
                            border: Border {
                                color: Color::BLACK,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .max_width(500),
            )
            .padding(50),
        );
        stack([self.base, contents]).into()
    }
}
