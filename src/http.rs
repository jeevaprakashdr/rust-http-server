use std::{
    clone,
    fmt::{Display, write},
    str::FromStr,
};

#[derive(Debug)]
pub(crate) enum HttpVerb {
    Get,
    Unknown(()),
}

impl FromStr for HttpVerb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let verb = match s.to_lowercase().as_str() {
            "get" => HttpVerb::Get,
            _ => HttpVerb::Unknown(()),
        };

        Ok(verb)
    }
}

#[derive(Debug)]
pub(crate) struct HttpRequest {
    method: HttpVerb,
    path: String,
    version: String,
}

impl HttpRequest {
    pub(crate) fn new(method: HttpVerb, path: String, version: String) -> Self {
        Self {
            method,
            path,
            version,
        }
    }

    pub(crate) fn path(&self) -> String {
        self.path.clone()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HttpStatusCode {
    Ok,
    NotFound,
}

impl From<HttpStatusCode> for u16 {
    fn from(value: HttpStatusCode) -> Self {
        match value {
            HttpStatusCode::Ok => 200,
            HttpStatusCode::NotFound => 404,
        }
    }
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            HttpStatusCode::Ok => "OK",
            HttpStatusCode::NotFound => "NOT FOUND",
        };

        write!(f, "{}", value)
    }
}

pub(crate) struct HttpResponse {
    status_code: HttpStatusCode,
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let crlf = "\r\n";
        let version = "HTTP/1.1";
        let status_code: u16 = self.status_code.into();
        let status_code_str = self.status_code.to_string();
        write!(f, "{version} {status_code} {status_code_str}{crlf}{crlf}")
    }
}

impl HttpResponse {
    pub(crate) fn with(status_code: HttpStatusCode) -> HttpResponse {
        Self { status_code }
    }
}

pub(crate) fn parse(request: &[u8]) -> Option<HttpRequest> {
    let mut request_parts = request.split(|&delimeter| delimeter == b'\n');

    let request_line = request_parts.next()?.trim_ascii();
    let request_line = request_line.split(|&c| c == b' ').collect::<Vec<_>>();

    let method = HttpVerb::from_str(str::from_utf8(request_line[0]).unwrap()).ok()?;
    let path = str::from_utf8(request_line[1]).ok()?;
    let version = str::from_utf8(request_line[2]).ok()?;

    Some(HttpRequest::new(
        method,
        path.to_string(),
        version.to_string(),
    ))
}
