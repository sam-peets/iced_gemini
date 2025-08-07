use iced::{Element, Font, Pixels, widget::text};

pub struct GeminiText {
    text: String,
    size: Option<Pixels>,
    font: Option<Font>,
}

impl GeminiText {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            size: None,
            font: None,
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn view<'a, Message>(self) -> Element<'a, Message> {
        let mut t = text(self.text).shaping(text::Shaping::Advanced);
        if let Some(size) = self.size {
            t = t.size(size);
        };

        if let Some(font) = self.font {
            t = t.font(font);
        }

        t.into()
    }
}
