use thiserror::Error;

pub mod gemtext;
pub mod response;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("unknown status: {0}")]
    UnknownStatus(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InputExpected = 10,
    SensitiveInput = 11,
    Success = 20,
    TemporaryRedirect = 30,
    PermanentRedirect = 31,
    TemporaryFailure = 40,
    ServerUnavailable = 41,
    CgiError = 42,
    ProxyError = 43,
    SlowDown = 44,
    PermanentFailure = 50,
    NotFound = 51,
    Gone = 52,
    ProxyRequestRefused = 53,
    BadRequest = 59,
    ClientCertificate = 60,
    CertificateNotAuthorized = 61,
    CertificateNotValid = 62,
}

impl TryFrom<u8> for Status {
    type Error = StatusError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(Status::InputExpected),
            11 => Ok(Status::SensitiveInput),
            20 => Ok(Status::Success),
            30 => Ok(Status::TemporaryRedirect),
            31 => Ok(Status::PermanentRedirect),
            40 => Ok(Status::TemporaryFailure),
            41 => Ok(Status::ServerUnavailable),
            42 => Ok(Status::CgiError),
            43 => Ok(Status::ProxyError),
            44 => Ok(Status::SlowDown),
            50 => Ok(Status::PermanentFailure),
            51 => Ok(Status::NotFound),
            52 => Ok(Status::Gone),
            53 => Ok(Status::ProxyRequestRefused),
            59 => Ok(Status::BadRequest),
            60 => Ok(Status::ClientCertificate),
            61 => Ok(Status::CertificateNotAuthorized),
            62 => Ok(Status::CertificateNotValid),
            10..=19 => Ok(Status::InputExpected),
            20..=29 => Ok(Status::Success),
            30..=39 => Ok(Status::TemporaryRedirect),
            40..=49 => Ok(Status::TemporaryFailure),
            50..=59 => Ok(Status::PermanentFailure),
            60..=69 => Ok(Status::ClientCertificate),
            _ => Err(StatusError::UnknownStatus(value)),
        }
    }
}
