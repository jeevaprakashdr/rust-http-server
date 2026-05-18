use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HttpVerb {
    Get,
    Post,
    Unknown(()),
}

impl FromStr for HttpVerb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let verb = match s.to_lowercase().as_str() {
            "get" => HttpVerb::Get,
            "post" => HttpVerb::Post,
            _ => HttpVerb::Unknown(()),
        };

        Ok(verb)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Encoding {
    Gzip,
    Unknown,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Encoding::Gzip => "gzip",
            Encoding::Unknown => "",
        };

        write!(f, "{val}")
    }
}

impl TryFrom<&[u8]> for Encoding {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"gzip" => Ok(Encoding::Gzip),
            _ => Err(()),
        }
    }
}

impl From<Encoding> for &[u8] {
    fn from(value: Encoding) -> Self {
        match value {
            Encoding::Gzip => b"gzip",
            Encoding::Unknown => b"",
        }
    }
}

impl FromStr for Encoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let encoding = match s.to_lowercase().as_str() {
            "gzip" => Encoding::Gzip,
            _ => Encoding::Unknown,
        };

        Ok(encoding)
    }
}

#[derive(Debug)]
pub(crate) struct HttpRequest<'a> {
    method: HttpVerb,
    path: Vec<&'a [u8]>,
    version: String,
    headers: HashMap<&'a [u8], &'a [u8]>,
    data: Vec<u8>,
}

impl<'a> HttpRequest<'a> {
    pub(crate) fn new(
        method: HttpVerb,
        path: Vec<&'a [u8]>,
        version: String,
        headers: HashMap<&'a [u8], &'a [u8]>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            data,
        }
    }

    pub(crate) fn get_path(&self) -> Vec<&[u8]> {
        self.path.clone()
    }

    pub(crate) fn get_headers(&self) -> HashMap<&'a [u8], &'a [u8]> {
        self.headers.clone()
    }

    pub(crate) fn get_method(&self) -> HttpVerb {
        self.method.clone()
    }

    pub(crate) fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub(crate) fn get_encoding(&self) -> Option<Encoding> {
        let encoding = self
            .headers
            .get::<&[u8]>(&HttpHeader::AcceptEncoding.into())
            .unwrap_or(&"".as_bytes())
            .trim_ascii();

        encoding
            .split(|&p| p == b',')
            .map(|bytes| Encoding::try_from(bytes.trim_ascii()))
            .find_map(|r| r.ok())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HttpStatusCode {
    Ok,
    Created,
    NotFound,
}

impl From<HttpStatusCode> for u16 {
    fn from(value: HttpStatusCode) -> Self {
        match value {
            HttpStatusCode::Ok => 200,
            HttpStatusCode::Created => 201,
            HttpStatusCode::NotFound => 404,
        }
    }
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            HttpStatusCode::Ok => "OK",
            HttpStatusCode::Created => "Created",
            HttpStatusCode::NotFound => "Not Found",
        };

        write!(f, "{}", value)
    }
}

pub(crate) enum HttpHeader {
    ContentType,
    ContentLength,
    UserAgent,
    AcceptEncoding,
    ContentEncoding,
}

impl From<HttpHeader> for String {
    fn from(value: HttpHeader) -> Self {
        match value {
            HttpHeader::ContentType => "Content-Type".to_string(),
            HttpHeader::ContentLength => "Content-Length".to_string(),
            HttpHeader::UserAgent => "User-Agent".to_string(),
            HttpHeader::AcceptEncoding => "Accept-Encoding".to_string(),
            HttpHeader::ContentEncoding => "Content-Encoding".to_string(),
        }
    }
}

impl From<HttpHeader> for &[u8] {
    fn from(value: HttpHeader) -> Self {
        match value {
            HttpHeader::ContentType => "Content-Type".as_bytes(),
            HttpHeader::ContentLength => "Content-Length".as_bytes(),
            HttpHeader::UserAgent => "User-Agent".as_bytes(),
            HttpHeader::AcceptEncoding => "Accept-Encoding".as_bytes(),
            HttpHeader::ContentEncoding => "Content-Encoding".as_bytes(),
        }
    }
}

pub(crate) struct HttpResponse {
    status_code: HttpStatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    pub(crate) fn with_status(status_code: HttpStatusCode) -> HttpResponse {
        Self {
            status_code,
            headers: HashMap::new(),
            body: Vec::<u8>::new(),
        }
    }

    pub(crate) fn with_header(&mut self, key: String, value: String) -> HttpResponse {
        self.headers.insert(key, value);
        Self {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: Vec::<u8>::new(),
        }
    }

    pub(crate) fn with_body(&mut self, body: Vec<u8>) -> HttpResponse {
        Self {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body,
        }
    }

    pub(crate) fn create(&self) -> Vec<u8> {
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
        let space = " ";

        let response = format!(
            "{version}{space}{status_code}{space}{status_code_str}{crlf}{headers}{crlf}{crlf}"
        );

        [response.as_bytes(), self.body.as_slice()].concat()
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
    let headers = get_headers(request_parts.clone().collect());
    let data = get_data(request_parts.collect());

    Some(HttpRequest::new(
        method,
        path,
        version.to_string(),
        headers,
        data,
    ))
}

fn get_data(request_parts: Vec<&[u8]>) -> Vec<u8> {
    request_parts.last().unwrap().to_vec()
}

fn get_headers(request_parts: Vec<&[u8]>) -> HashMap<&[u8], &[u8]> {
    let mut headers = HashMap::<&[u8], &[u8]>::new();
    for header_line in request_parts {
        let kv = header_line.split(|&p| p == b':').collect::<Vec<_>>();

        if kv.is_empty() {
            continue;
        }

        let key = kv.first().unwrap();
        let value = kv.last().unwrap();
        headers.insert(key, value);
    }

    headers
}
