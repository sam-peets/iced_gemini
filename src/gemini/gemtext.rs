use iced::{
    Border, Element, Font, padding,
    widget::{Column, Container, button, container, text, tooltip},
};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct Document {
    pub lines: Vec<Line>,
    pub url: Url,
}

impl Document {
    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_press: fn(&Line) -> Message,
    ) -> Element<'a, Message> {
        Column::from_vec(self.lines.iter().map(|line| line.view(on_press)).collect()).into()
    }
    pub fn parse(url: &Url, doc: &str) -> anyhow::Result<Self> {
        let mut state = ParserMode::Normal;
        let mut lines = Vec::new();
        let mut doc = doc;
        let mut acc = String::new();
        while let Some((line, rest)) = doc.split_once("\n") {
            if state == ParserMode::Normal {
                let l = Line::parse(url, line)?;
                if let Line::Toggle(s) = l {
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
                acc.push_str(&format!("{line}\n"));
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
    #[error("parsing error: ${0}")]
    Other(String),
}
impl Line {
    fn view_link<'a, Message: Clone + 'a>(
        &'a self,
        url: &'a Url,
        friendly: Option<&'a str>,
        on_press: fn(&Line) -> Message,
    ) -> Element<'a, Message> {
        tooltip(
            button(text(friendly.unwrap_or(url.as_str())).shaping(text::Shaping::Advanced))
                .on_press_with(move || on_press(&self.clone())),
            text(url.to_string()).shaping(text::Shaping::Advanced),
            tooltip::Position::Right,
        )
        .into()
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_press: fn(&Line) -> Message,
    ) -> Element<'a, Message> {
        let sizes = [40, 30, 20];
        match self {
            Line::Text(s) => Element::new(text(s).shaping(text::Shaping::Advanced)),
            Line::Link(url, friendly) => self.view_link(url, friendly.as_deref(), on_press),
            Line::Heading(level, s) => Element::new(
                text(s)
                    .size(sizes[*level - 1])
                    .shaping(text::Shaping::Advanced),
            ),
            Line::List(s) => Element::new(text(format!(" • {s}")).shaping(text::Shaping::Advanced)),
            Line::Quote(s) => Element::new(
                Container::new(text(s).shaping(text::Shaping::Advanced)).padding(padding::left(10)),
            ),
            Line::PreFormatted(s) => Element::new(
                text(s)
                    .font(Font::MONOSPACE)
                    .shaping(text::Shaping::Advanced),
            ),
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

    fn parse_header(line: &str) -> anyhow::Result<Self> {
        let hashes = line.chars().take_while(|&c| c == '#').count();
        let rest = line[hashes..].trim_start();
        Ok(Line::Heading(hashes, rest.to_string()))
    }

    fn parse_list(line: &str) -> anyhow::Result<Self> {
        let (_, l) = line
            .split_once(char::is_whitespace)
            .ok_or(LineParsingError::Other("No text after bullet".to_string()))?;
        Ok(Line::List(l.to_string()))
    }

    fn parse_toggle(line: &str) -> anyhow::Result<Self> {
        Ok(Line::Toggle(line[3..].to_string()))
    }

    fn parse_quote(line: &str) -> anyhow::Result<Self> {
        let (_, l) = line
            .split_once(char::is_whitespace)
            .ok_or(LineParsingError::Other("No text after bullet".to_string()))?;
        Ok(Line::Quote(l.to_string()))
    }

    pub fn parse(current_url: &Url, line: &str) -> anyhow::Result<Self> {
        log::trace!("Line: parsing {line}");
        match line {
            x if x.starts_with("=>") => Line::parse_link(current_url, line),
            x if x.starts_with("#") => Line::parse_header(line),
            x if x.starts_with("*") => Line::parse_list(line),
            x if x.starts_with("```") => Line::parse_toggle(line),
            x if x.starts_with(">") => Line::parse_quote(line),
            x => Ok(Line::Text(x.to_string())),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParserMode {
    Normal,
    PreFormatted,
}
