use iced::{
    Color, Element,
    widget::{button, text},
};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct Document {
    pub lines: Vec<Line>,
    pub url: Url,
}

impl Document {
    pub fn parse(url: &Url, doc: &str) -> anyhow::Result<Self> {
        let state = ParserMode::Normal;
        let mut lines = Vec::new();
        let mut doc = doc;
        while let Some((line, rest)) = doc.split_once("\n") {
            lines.push(Line::parse(url, line)?);
            doc = rest;
        }

        Ok(Document {
            lines,
            url: url.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Text(String),
    Link(Url, Option<String>), // URL, friendly name
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
        url: &Url,
        friendly: &Option<String>,
        on_press: fn(&Line) -> Message,
    ) -> Element<'a, Message> {
        button(text(friendly.clone().unwrap_or(url.to_string())))
            .on_press_with(move || on_press(&self.clone()))
            .into()
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_press: fn(&Line) -> Message,
    ) -> Element<'a, Message> {
        match self {
            Line::Text(s) => Element::new(text(s)),
            Line::Link(url, friendly) => self.view_link(url, friendly, on_press),
            Line::Heading(level, s) => Element::new(text(s).size(40.0 / *level as f32)),
            Line::List() => todo!(),
            Line::Quote() => todo!(),
            Line::Toggle() => todo!(),
        }
    }
}

impl Line {
    fn parse_link(current_url: &Url, line: &str) -> anyhow::Result<Self> {
        let mut spl = line.splitn(3, char::is_whitespace);
        spl.next(); // skip =>
        let uri = spl
            .next()
            .ok_or(LineParsingError::MissingUri)?
            .trim()
            .to_string();

        let friendly = spl.next().map(|x| x.trim().to_string());
        Ok(Line::Link(current_url.join(&uri)?, friendly))
    }

    fn parse_header(line: &str) -> anyhow::Result<Self> {
        let hashes = line.chars().take_while(|&c| c == '#').count();
        let rest = line[hashes..].trim_start();
        Ok(Line::Heading(hashes, rest.to_string()))
    }

    pub fn parse(current_url: &Url, line: &str) -> anyhow::Result<Self> {
        log::trace!("Line: parsing {line}");
        match line {
            x if x.starts_with("=>") => Line::parse_link(current_url, line),
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
