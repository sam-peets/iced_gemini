use iced::{
    Color, Element,
    widget::{button, text},
};
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
    Heading(usize, String),
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
    fn view_link<'a, Message: Clone + 'a>(
        &'a self,
        uri: &String,
        friendly: &Option<String>,
    ) -> Element<'a, Message> {
        button(text(friendly.clone().unwrap_or(uri.to_string()))).into()
    }

    pub fn view<'a, Message: Clone + 'a>(&'a self) -> Element<'a, Message> {
        match self {
            Line::Text(s) => Element::new(text(s)),
            Line::Link(uri, friendly) => self.view_link(uri, friendly),
            Line::Heading(level, s) => Element::new(text(s).size(40.0 / *level as f32)),
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

    fn parse_header(line: &str) -> anyhow::Result<Self> {
        let hashes = line.chars().take_while(|&c| c == '#').count();
        let rest = line[hashes..].trim_start();
        Ok(Line::Heading(hashes, rest.to_string()))
    }

    pub fn parse(line: &str) -> anyhow::Result<Self> {
        log::trace!("Line: parsing {line}");
        match line {
            x if x.starts_with("=>") => Line::parse_link(line),
            x if x.starts_with("#") => Line::parse_header(line),
            x => Ok(Line::Text(x.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
enum ParserMode {
    Normal,
    PreFormatted,
}
