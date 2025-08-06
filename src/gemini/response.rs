use std::str::FromStr;

use thiserror::Error;

use crate::gemini::Status;

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("parsing error: `{0}`")]
    ParsingError(String),
}
#[derive(Debug, Clone)]
pub struct Response {
    pub status: Status,
    pub ctx: Option<String>,
    pub body: Option<String>,
}

impl FromStr for Response {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (status, body) = s.split_once("\r\n").ok_or(ResponseError::ParsingError(
            "no CRLF in response".to_string(),
        ))?;

        let mut spl = status.splitn(2, ' ');

        let status = spl
            .next()
            .and_then(|s| s.parse::<u8>().ok())
            .and_then(|s| Status::try_from(s).ok())
            .ok_or_else(|| ResponseError::ParsingError("invalid status code".to_string()))?;

        Ok(Response {
            status,
            ctx: spl.next().map(|s| s.to_string()),
            body: Some(body.to_string()),
        })
    }
}
