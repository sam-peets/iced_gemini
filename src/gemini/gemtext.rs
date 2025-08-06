use iced::{Color, Element, widget::text};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub lines: Vec<Line>,
}

impl Document {
    pub fn parse(doc: &str) -> anyhow::Result<Self> {
        let state = ParserMode::Normal;
        let mut lines = Vec::new();
        let mut doc = doc;
        while let Some((line, rest)) = doc.split_once("\n") {
            lines.push(Line::parse(line)?);
            doc = rest;
        }

        Ok(Document { lines })
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Text(String),
    Link(String, Option<String>), // URL, friendly name
    Heading(isize, String),
    List(),
    Quote(),
    Toggle(),
}

#[derive(Error, Debug)]
pub enum LineParsingError {
    #[error("malformed link line: missing URI")]
    MissingUri,
}
impl Line {
    fn view_link<Message>(&self, uri: &String, friendly: &Option<String>) -> Element<'_, Message> {
        text(friendly.clone().unwrap_or(uri.to_string()))
            .color(Color::from_rgb(0.1, 0.1, 0.8))
            .into()
    }

    pub fn view<Message>(&self) -> Element<'_, Message> {
        match self {
            Line::Text(s) => Element::new(text(s)),
            Line::Link(uri, friendly) => self.view_link(uri, friendly),
            Line::Heading(_, _) => todo!(),
            Line::List() => todo!(),
            Line::Quote() => todo!(),
            Line::Toggle() => todo!(),
        }
        .into()
    }
}

impl Line {
    fn parse_link(line: &str) -> anyhow::Result<Self> {
        let mut spl = line.splitn(3, ' ');
        spl.next(); // skip =>
        Ok(Line::Link(
            spl.next().ok_or(LineParsingError::MissingUri)?.to_string(),
            spl.next().and_then(|x| Some(x.to_string())),
        ))
    }

    pub fn parse(line: &str) -> anyhow::Result<Self> {
        log::trace!("Line: parsing {line}");
        match line {
            x if x.starts_with("=>") => Line::parse_link(line),
            x => Ok(Line::Text(x.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
enum ParserMode {
    Normal,
    PreFormatted,
}
