use std::{
    collections::HashMap, fmt::Display, str::FromStr
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
pub(crate) struct HttpRequest<'a> {
    method: HttpVerb,
    path: Vec<&'a [u8]>,
    version: String,
}

impl<'a> HttpRequest<'a> {
    pub(crate) fn new(method: HttpVerb, path: Vec<&'a [u8]>, version: String) -> Self {
        Self {
            method,
            path,
            version,
        }
    }

    pub(crate) fn path(&self) -> Vec<&[u8]> {
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
            HttpStatusCode::NotFound => "Not Found",
        };

        write!(f, "{}", value)
    }
}

pub(crate) enum HttpResponseHeader {
    ContentType,
    ContentLength,
}

impl From<HttpResponseHeader> for String {
    fn from(value: HttpResponseHeader) -> Self {
        match value {
            HttpResponseHeader::ContentType => "Content-Type".to_string(),
            HttpResponseHeader::ContentLength => "Content-Length".to_string(),
        }
    }
}

pub(crate) struct HttpResponse<'a> {
    status_code: HttpStatusCode,
    headers: HashMap<String, String>,
    body: &'a [u8],
}

impl<'a> Display for HttpResponse<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let crlf = "\r\n";
        let version = "HTTP/1.1";
        let status_code: u16 = self.status_code.into();
        let status_code_str = self.status_code.to_string();
        let headers = self
            .headers
            .iter()
            .map(|h| format!("{}: {}", h.0, h.1))
            .collect::<Vec<_>>()
            .join("\r\n");
        let body = str::from_utf8(self.body).unwrap_or("");
        let space = " ";
        write!(
            f,
            "{version}{space}{status_code}{space}{status_code_str}{crlf}{headers}{crlf}{crlf}{body}"
        )
    }
}

impl<'a> HttpResponse<'a> {
    pub(crate) fn with_status(status_code: HttpStatusCode) -> HttpResponse<'a> {
        Self {
            status_code,
            headers: HashMap::new(),
            body: b"",
        }
    }

    pub(crate) fn with_header(&mut self, key: String, value: String) -> HttpResponse<'a> {
        self.headers.insert(key, value);
        Self {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: b"",
        }
    }

    pub(crate) fn with_body(&mut self, body: &'a [u8]) -> HttpResponse<'a> {
        Self {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body,
        }
    }
}

pub(crate) fn parse<'a>(request: &'a [u8]) -> Option<HttpRequest<'a>> {
    let mut request_parts = request.split(|&delimeter| delimeter == b'\n');

    let request_line = request_parts.next()?.trim_ascii();
    let request_line = request_line.split(|&c| c == b' ').collect::<Vec<_>>();

    let method = HttpVerb::from_str(str::from_utf8(request_line[0]).unwrap()).ok()?;
    let path: Vec<&[u8]> = request_line[1]
        .split(|&p| p == b'/')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    let version = str::from_utf8(request_line[2]).ok()?;

    Some(HttpRequest::new(method, path, version.to_string()))
}
