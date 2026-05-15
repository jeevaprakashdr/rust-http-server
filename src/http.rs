use std::str::FromStr;

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

pub(crate) struct HttpResponse {
    
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
