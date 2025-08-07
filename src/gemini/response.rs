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
    pub body: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for Response {
    type Error = ResponseError;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let mut spl = s.splitn(2, |x| *x == b'\n');
        let status = String::from_utf8(
            spl.next()
                .ok_or(ResponseError::ParsingError(
                    "No status line in response".into(),
                ))?
                .to_vec(),
        )
        .map_err(|e| ResponseError::ParsingError(e.to_string()))?;
        let body = spl
            .next()
            .ok_or(ResponseError::ParsingError("No body in response".into()))?;
        let mut spl = status.splitn(2, ' ');

        let status = spl
            .next()
            .and_then(|s| s.parse::<u8>().ok())
            .and_then(|s| Status::try_from(s).ok())
            .ok_or_else(|| ResponseError::ParsingError("invalid status code".to_string()))?;

        Ok(Response {
            status,
            ctx: spl.next().map(|s| s.trim().to_string()),
            body: Some(body.to_vec()),
        })
    }
}
