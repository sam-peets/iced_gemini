use iced::{Element, widget::text};

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
    Link(http::Uri, Option<String>), // URL, friendly name
    Heading(isize, String),
    List(),
    Quote(),
    Toggle(),
}

impl Line {
    pub fn view<Message>(&self) -> Element<'_, Message> {
        match self {
            Line::Text(s) => text(s),
            Line::Link(uri, friendly) => text(friendly.clone().unwrap_or(uri.to_string())),
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
            spl.next().unwrap().parse()?,
            spl.next().and_then(|x| Some(x.to_string())),
        ))
    }

    pub fn parse(line: &str) -> anyhow::Result<Self> {
        log::info!("Line: parsing {line}");
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
