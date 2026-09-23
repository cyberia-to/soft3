use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;

use cybergraph::native;

use super::Node;

const MAX_HEADERS: usize = 64 * 1024;
const MAX_BODY: usize = 8 * 1024 * 1024;

pub(super) struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub idempotency_key: Option<String>,
    pub body: Vec<u8>,
}

#[cfg_attr(test, derive(Debug))]
pub(super) struct Response {
    status: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    /// Inspect a response body without a socket — requests.rs's submit() tests.
    #[cfg(test)]
    pub(super) fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap()
    }

    pub fn text(body: impl Into<String>) -> Self {
        Self::bytes("text/plain; charset=utf-8", body.into().into_bytes())
    }

    pub fn bytes(content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status: "200 OK",
            content_type,
            body,
        }
    }

    fn write(self, stream: &mut TcpStream) -> io::Result<()> {
        write!(stream, "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n", self.status, self.content_type, self.body.len())?;
        stream.write_all(&self.body)
    }

    /// Inspect a response without a socket — routes.rs's dispatch tests.
    #[cfg(test)]
    pub(super) fn parts(&self) -> (&str, &str, &[u8]) {
        (self.status, self.content_type, &self.body)
    }
}

#[cfg_attr(test, derive(Debug))]
pub(super) struct Error {
    pub status: &'static str,
    pub code: &'static str,
    pub message: String,
}

impl Error {
    pub fn bad(message: impl Into<String>) -> Self {
        Self {
            status: "400 Bad Request",
            code: "invalid_request",
            message: message.into(),
        }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: "503 Service Unavailable",
            code: "storage_unavailable",
            message: message.into(),
        }
    }

    fn response(self) -> Response {
        Response {
            status: self.status,
            content_type: "application/json",
            body: serde_json::json!({"error": self.message, "code": self.code})
                .to_string()
                .into_bytes(),
        }
    }
}

impl From<native::Error> for Error {
    fn from(error: native::Error) -> Self {
        let (status, code) = match &error {
            native::Error::Conflict => ("409 Conflict", "request_conflict"),
            native::Error::Invalid(_) => ("400 Bad Request", "operation_rejected"),
            native::Error::Limit(_) => ("400 Bad Request", "operation_limit"),
            native::Error::Unsupported(_) => {
                ("422 Unprocessable Content", "unsupported_representation")
            }
            native::Error::Storage(_) | native::Error::Corrupt(_) => {
                ("503 Service Unavailable", "storage_unavailable")
            }
        };
        Self {
            status,
            code,
            message: error.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        if matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ) {
            Self {
                status: "408 Request Timeout",
                code: "request_timeout",
                message: error.to_string(),
            }
        } else {
            Self::bad(error.to_string())
        }
    }
}

pub(super) fn handle_client(mut stream: TcpStream, node: &Mutex<Node>) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    let response = read_request(&mut stream)
        .and_then(|request| super::routes::route(node, request))
        .unwrap_or_else(Error::response);
    response.write(&mut stream)
}

fn read_request(stream: &mut impl Read) -> Result<Request, Error> {
    let mut raw = Vec::with_capacity(8192);
    let mut buffer = [0u8; 8192];
    let header_end = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(Error::bad("incomplete request headers"));
        }
        raw.extend_from_slice(&buffer[..read]);
        if let Some(position) = raw.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            if position + 4 > MAX_HEADERS {
                return Err(Error::bad("headers exceed 64 KiB"));
            }
            break position + 4;
        }
        if raw.len() > MAX_HEADERS {
            return Err(Error::bad("headers exceed 64 KiB"));
        }
    };
    let head = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| Error::bad("invalid header encoding"))?;
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next().unwrap_or("").split(' ');
    let method = request_line.next().unwrap_or("").to_owned();
    let target = request_line.next().unwrap_or("");
    let version = request_line.next().unwrap_or("");
    if !matches!(version, "HTTP/1.0" | "HTTP/1.1")
        || request_line.next().is_some()
        || !target.starts_with('/')
        || !target.bytes().all(|b| b.is_ascii_graphic())
        || method.is_empty()
        || !method.bytes().all(|b| b.is_ascii_uppercase())
    {
        return Err(Error::bad("invalid request line"));
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let (path, query) = (path.to_owned(), query.to_owned());
    let mut content_length = None;
    let mut idempotency_key = None;
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| Error::bad("invalid header"))?;
        if name.is_empty()
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || value.bytes().any(|b| b.is_ascii_control() && b != b'\t')
        {
            return Err(Error::bad("invalid header"));
        }
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(Error::bad(
                "transfer encoding is unsupported; use Content-Length",
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some()
                || value.is_empty()
                || !value.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(Error::bad("invalid or duplicate Content-Length"));
            }
            content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| Error::bad("invalid Content-Length"))?,
            );
        }
        if name.eq_ignore_ascii_case("idempotency-key")
            && idempotency_key.replace(value.to_owned()).is_some()
        {
            return Err(Error::bad("duplicate Idempotency-Key"));
        }
    }
    if method == "POST" && content_length.is_none() {
        return Err(Error::bad("POST requires Content-Length"));
    }
    let content_length = content_length.unwrap_or(0);
    if content_length > MAX_BODY {
        return Err(Error {
            status: "413 Content Too Large",
            code: "request_too_large",
            message: "body exceeds 8 MiB".into(),
        });
    }
    let body_end = header_end + content_length;
    if raw.len() > body_end {
        return Err(Error::bad("unexpected bytes after request body"));
    }
    while raw.len() < body_end {
        let wanted = (body_end - raw.len()).min(buffer.len());
        let read = stream.read(&mut buffer[..wanted])?;
        if read == 0 {
            return Err(Error::bad("incomplete request body"));
        }
        raw.extend_from_slice(&buffer[..read]);
    }
    Ok(Request {
        method,
        path,
        query,
        idempotency_key,
        body: raw.split_off(header_end),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(raw: &[u8]) -> Request {
        read_request(&mut &raw[..])
            .unwrap_or_else(|e| panic!("expected ok, got {} {}", e.status, e.message))
    }

    fn err(raw: &[u8]) -> Error {
        read_request(&mut &raw[..]).err().expect("expected error")
    }

    #[test]
    fn parses_simple_get() {
        let req = ok(b"GET /v1/link HTTP/1.1\r\nHost: x\r\n\r\n");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/v1/link");
        assert_eq!(req.query, "");
        assert!(req.idempotency_key.is_none());
        assert!(req.body.is_empty());
    }

    #[test]
    fn splits_path_and_query() {
        let req = ok(b"GET /v2/history?after=5&limit=10 HTTP/1.1\r\n\r\n");
        assert_eq!(req.path, "/v2/history");
        assert_eq!(req.query, "after=5&limit=10");
    }

    #[test]
    fn parses_post_with_body() {
        let raw = b"POST /v1/link HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
        let req = ok(raw);
        assert_eq!(req.method, "POST");
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn captures_idempotency_key() {
        let raw = b"POST /v1/pay HTTP/1.1\r\nContent-Length: 0\r\nIdempotency-Key: abc-123\r\n\r\n";
        let req = ok(raw);
        assert_eq!(req.idempotency_key.as_deref(), Some("abc-123"));
    }

    #[test]
    fn post_without_content_length_is_rejected() {
        let e = err(b"POST /v1/link HTTP/1.1\r\n\r\n");
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_unsupported_http_version() {
        let e = err(b"GET / HTTP/2.0\r\n\r\n");
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_lowercase_method() {
        let e = err(b"get / HTTP/1.1\r\n\r\n");
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_extra_token_in_request_line() {
        let e = err(b"GET / HTTP/1.1 extra\r\n\r\n");
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_non_graphic_byte_in_target() {
        let e = err(b"GET /a\tb HTTP/1.1\r\n\r\n");
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_duplicate_content_length() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n";
        let e = err(raw);
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_duplicate_idempotency_key() {
        let raw =
            b"POST / HTTP/1.1\r\nContent-Length: 0\r\nIdempotency-Key: a\r\nIdempotency-Key: b\r\n\r\n";
        let e = err(raw);
        assert_eq!(e.status, "400 Bad Request");
    }

    #[test]
    fn rejects_transfer_encoding() {
        let raw = b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n";
        let e = err(raw);
        assert_eq!(
            e.message,
            "transfer encoding is unsupported; use Content-Length"
        );
    }

    #[test]
    fn rejects_content_length_over_max_body() {
        let raw = format!("POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n", MAX_BODY + 1);
        let e = err(raw.as_bytes());
        assert_eq!(e.status, "413 Content Too Large");
    }

    #[test]
    fn rejects_unexpected_bytes_after_body() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\nhello";
        let e = err(raw);
        assert_eq!(e.message, "unexpected bytes after request body");
    }

    #[test]
    fn rejects_incomplete_headers() {
        let e = err(b"GET / HTTP/1.1\r\n");
        assert_eq!(e.message, "incomplete request headers");
    }

    #[test]
    fn rejects_headers_exceeding_max_size() {
        let mut raw = b"GET / HTTP/1.1\r\n".to_vec();
        raw.extend(std::iter::repeat_n(b'a', MAX_HEADERS + 10));
        let e = err(&raw);
        assert_eq!(e.message, "headers exceed 64 KiB");
    }
}
