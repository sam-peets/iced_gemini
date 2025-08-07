use iced::{
    Element, Font, padding,
    widget::{Column, Container},
};
use std::fmt::Write;
use thiserror::Error;
use url::Url;

use crate::ui::{gemini_link::GeminiLink, gemini_text::GeminiText};

#[derive(Debug, Clone)]
pub struct Document {
    pub lines: Vec<Line>,
    pub url: Url,
}

impl Document {
    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_press_link: fn(&Url) -> Message,
    ) -> Element<'a, Message> {
        Column::from_vec(
            self.lines
                .iter()
                .map(|line| line.view(on_press_link))
                .collect(),
        )
        .into()
    }
    pub fn parse(url: &Url, doc: &str) -> anyhow::Result<Self> {
        let mut state = ParserMode::Normal;
        let mut lines = Vec::new();
        let mut doc = doc;
        let mut acc = String::new();
        while let Some((line, rest)) = doc.split_once('\n') {
            if state == ParserMode::Normal {
                let l = Line::parse(url, line)?;
                if let Line::Toggle(s) = l {
                    log::info!("Document::parse: read toggle: {s}");
                    state = if state == ParserMode::Normal {
                        acc = String::new();
                        ParserMode::PreFormatted
                    } else {
                        ParserMode::Normal
                    }
                } else {
                    lines.push(l);
                }
            } else if line.starts_with("```") {
                lines.push(Line::PreFormatted(acc.clone()));
                state = ParserMode::Normal;
            } else {
                writeln!(acc, "{line}")?;
            }

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
    List(String),
    Quote(String),
    Toggle(String),
    PreFormatted(String),
    Image(iced::advanced::image::Handle),
}

#[derive(Error, Debug)]
pub enum LineParsingError {
    #[error("malformed link line: missing URI")]
    MissingUri,
}
impl Line {
    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_press_link: fn(&Url) -> Message,
    ) -> Element<'a, Message> {
        let sizes = [40, 30, 20];
        match self {
            Line::Text(s) => GeminiText::new(s).view(),
            Line::Link(url, friendly) => {
                GeminiLink::new(url.clone(), friendly.clone(), on_press_link).view()
            }
            Line::Heading(level, s) => GeminiText::new(s).size(sizes[*level - 1]).view(),
            Line::List(s) => GeminiText::new(&format!(" • {s}")).view(),
            Line::Quote(s) => {
                Element::new(Container::new(GeminiText::new(s).view()).padding(padding::left(10)))
            }
            Line::PreFormatted(s) => GeminiText::new(s).font(Font::MONOSPACE).view(),
            Line::Toggle(_) => unreachable!(),
            Line::Image(handle) => Element::new(iced::widget::image(handle)),
        }
    }

    fn parse_link(current_url: &Url, line: &str) -> anyhow::Result<Self> {
        let line = &line[2..]; // we don't care about the =>
        let mut spl = line.trim().splitn(2, char::is_whitespace);
        let uri = spl
            .next()
            .ok_or(LineParsingError::MissingUri)?
            .trim()
            .to_string();

        let friendly = spl.next().map(|x| x.trim().to_string());
        log::info!("{uri} {friendly:?}");
        Ok(Line::Link(current_url.join(&uri)?, friendly))
    }

    fn parse_header(line: &str) -> Self {
        let hashes = line.chars().take_while(|&c| c == '#').count();
        let rest = line[hashes..].trim_start();
        Line::Heading(hashes, rest.to_string())
    }

    fn parse_list(line: &str) -> Self {
        let line = line[1..].trim(); // everything after the `*`
        Line::List(line.to_string())
    }

    fn parse_toggle(line: &str) -> Self {
        Line::Toggle(line[3..].to_string())
    }

    fn parse_quote(line: &str) -> Self {
        let line = line[1..].trim(); // everything after the `>`
        Line::Quote(line.to_string())
    }

    pub fn parse(current_url: &Url, line: &str) -> anyhow::Result<Self> {
        log::trace!("Line: parsing {line}");
        match line {
            x if x.starts_with("=>") => Line::parse_link(current_url, line),
            x if x.starts_with('#') => Ok(Line::parse_header(line)),
            x if x.starts_with('*') => Ok(Line::parse_list(line)),
            x if x.starts_with("```") => Ok(Line::parse_toggle(line)),
            x if x.starts_with('>') => Ok(Line::parse_quote(line)),
            x => Ok(Line::Text(x.to_string())),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParserMode {
    Normal,
    PreFormatted,
}
