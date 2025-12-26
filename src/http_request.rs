use std::{
    collections::{HashMap, hash_map},
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader},
    net::TcpStream,
};

#[derive(Debug)]
pub struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: String, path: String, headers: HashMap<String, String>) -> HttpRequest {
        HttpRequest {
            method,
            path,
            headers,
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn headers(&'_ self) -> hash_map::Iter<'_, String, String> {
        self.headers.iter()
    }

    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

impl TryFrom<&TcpStream> for HttpRequest {
    type Error = HttpRequestParseError;
    fn try_from(stream: &TcpStream) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(stream).lines();

        let status_line = reader
            .next()
            .ok_or(HttpRequestParseError(
                "Invalid request: Missing HTTP status line.".to_owned(),
            ))?
            .map_err(|error| {
                HttpRequestParseError(format!(
                    "TcpStream error. Could not read from TcpStream\nInner: {error}"
                ))
            })?;

        let mut status_line_iter = status_line.trim().split(" ");

        let method = status_line_iter.next().ok_or(HttpRequestParseError(
            "Invalid request: Missing method".to_owned(),
        ))?;
        let path = status_line_iter.next().ok_or(HttpRequestParseError(
            "Invalid request: Missing path".to_owned(),
        ))?;

        let mut headers = HashMap::new();
        while let Some(line) = reader.next() {
            let line = line.map_err(|error| {
                HttpRequestParseError(format!(
                    "TcpStream error. Could not read from TcpStream\nInner: {error}"
                ))
            })?;

            // Ending of headers, skip parsing them.
            if line.is_empty() {
                break;
            }

            let mut header_it = line.split(":");
            let name = header_it.next().ok_or(HttpRequestParseError(
                "Invalid request: Empty header name.".to_owned(),
            ))?;
            let value = header_it.next().ok_or(HttpRequestParseError(
                "Invalid request: Empty header value.".to_owned(),
            ))?;

            headers.insert(name.to_owned(), value.to_owned());
        }

        Ok(HttpRequest::new(
            method.to_owned(),
            path.to_owned(),
            headers,
        ))
    }
}

#[derive(Debug)]
pub struct HttpRequestParseError(String);

impl Display for HttpRequestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let HttpRequestParseError(error) = self;
        f.write_str(&format!("Error wile parsing HttpRequest: {}", error))
    }
}

impl Error for HttpRequestParseError {}
