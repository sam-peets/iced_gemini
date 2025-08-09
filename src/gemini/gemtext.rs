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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quote() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "> hello")?;
        if let Line::Quote(q) = x {
            assert_eq!(q, "hello");
        } else {
            panic!()
        }

        Ok(())
    }
    #[test]
    fn test_parse_quote_no_space() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, ">hello")?;
        if let Line::Quote(q) = x {
            assert_eq!(q, "hello");
        } else {
            panic!()
        }

        Ok(())
    }
    #[test]
    fn test_parse_quote_no_text() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, ">")?;
        if let Line::Quote(q) = x {
            assert_eq!(q, "");
        } else {
            panic!()
        }
        Ok(())
    }

    #[test]
    fn test_parse_list() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "* hello")?;
        if let Line::List(q) = x {
            assert_eq!(q, "hello");
        } else {
            panic!()
        }
        Ok(())
    }

    #[test]
    fn test_parse_list_no_space() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "*hello")?;
        if let Line::List(q) = x {
            assert_eq!(q, "hello");
        } else {
            panic!()
        }
        Ok(())
    }

    #[test]
    fn test_parse_list_no_text() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "*")?;
        if let Line::List(q) = x {
            assert_eq!(q, "");
        } else {
            panic!()
        }
        Ok(())
    }

    #[test]
    fn test_parse_link() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let target = Url::parse("gemini://example2.com/")?;
        let x = Line::parse(&url, "=> gemini://example2.com/")?;
        if let Line::Link(link_url, friendly) = x {
            assert_eq!(link_url, target);
            assert_eq!(friendly, None);
        } else {
            panic!();
        }
        let x = Line::parse(&url, "=> gemini://example2.com/ hello")?;
        if let Line::Link(link_url, friendly) = x {
            let friendly = friendly.unwrap();
            assert_eq!(link_url, target);
            assert_eq!(friendly, "hello");
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn test_parse_link_relative() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let target = Url::parse("gemini://example.com/relative")?;
        let x = Line::parse(&url, "=> relative")?;
        if let Line::Link(link_url, friendly) = x {
            assert_eq!(link_url, target);
            assert_eq!(friendly, None);
        } else {
            panic!();
        }

        let x = Line::parse(&url, "=> relative friendly")?;
        if let Line::Link(link_url, friendly) = x {
            let friendly = friendly.unwrap();
            assert_eq!(link_url, target);
            assert_eq!(friendly, "friendly");
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn test_parse_link_relative_deep() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/1/2/3")?;
        let target = Url::parse("gemini://example.com/1/2/abc/xyz")?;
        let x = Line::parse(&url, "=> abc/xyz")?;
        if let Line::Link(link_url, friendly) = x {
            assert_eq!(link_url, target);
            assert_eq!(friendly, None);
        } else {
            panic!();
        }

        let x = Line::parse(&url, "=> abc/xyz friendly")?;
        if let Line::Link(link_url, friendly) = x {
            let friendly = friendly.unwrap();
            assert_eq!(link_url, target);
            assert_eq!(friendly, "friendly");
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn test_parse_link_relative_root() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/1/2/3")?;
        let target = Url::parse("gemini://example.com/abc/xyz")?;
        let x = Line::parse(&url, "=> /abc/xyz")?;
        if let Line::Link(link_url, friendly) = x {
            assert_eq!(link_url, target);
            assert_eq!(friendly, None);
        } else {
            panic!();
        }

        let x = Line::parse(&url, "=> /abc/xyz friendly")?;
        if let Line::Link(link_url, friendly) = x {
            let friendly = friendly.unwrap();
            assert_eq!(link_url, target);
            assert_eq!(friendly, "friendly");
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn test_parse_link_friendly_space() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let target = Url::parse("gemini://example.com/abc")?;
        let x = Line::parse(&url, "=> abc long friendly text with spaces")?;
        if let Line::Link(link_url, friendly) = x {
            let friendly = friendly.unwrap();
            assert_eq!(link_url, target);
            assert_eq!(friendly, "long friendly text with spaces");
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn test_parse_header() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "# hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 1);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        let x = Line::parse(&url, "## hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 2);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        let x = Line::parse(&url, "### hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 3);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        Ok(())
    }

    #[test]
    fn test_parse_header_no_space() -> anyhow::Result<()> {
        let url = Url::parse("gemini://example.com/")?;
        let x = Line::parse(&url, "#hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 1);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        let x = Line::parse(&url, "##hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 2);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        let x = Line::parse(&url, "###hello")?;
        if let Line::Heading(level, s) = x {
            assert_eq!(level, 3);
            assert_eq!(s, "hello");
        } else {
            panic!();
        }

        Ok(())
    }
}
