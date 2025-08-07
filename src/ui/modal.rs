use iced::{
    Border, Color, Element, border, color,
    widget::{center, container, mouse_area, opaque, stack},
};

pub struct Modal<'a, Message: Clone + 'a> {
    base: Element<'a, Message>,
    contents: Element<'a, Message>,
}

impl<'a, Message: Clone + 'a> Modal<'a, Message> {
    pub fn new(base: Element<'a, Message>, contents: Element<'a, Message>) -> Self {
        Self { base, contents }
    }

    pub fn view(self) -> Element<'a, Message> {
        let contents = opaque(center(container(self.contents).style(|t| {
            let palette = t.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })));
        stack([self.base, contents]).into()
    }
}
