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
    body_stream: BufReader<TcpStream>,
}

impl HttpRequest {
    pub fn new(
        method: String,
        path: String,
        headers: HashMap<String, String>,
        body_stream: BufReader<TcpStream>,
    ) -> HttpRequest {
        HttpRequest {
            method,
            path,
            headers,
            body_stream,
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

    pub fn body_stream(&self) -> &BufReader<TcpStream> {
        &self.body_stream
    }
}

impl TryFrom<TcpStream> for HttpRequest {
    type Error = HttpRequestParseError;

    fn try_from(stream: TcpStream) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(stream);

        let mut status_line = String::new();
        reader
            .read_line(&mut status_line)
            .map_err(|error| HttpRequestParseError::StreamError(error))?;

        if status_line.trim().is_empty() {
            return Err(HttpRequestParseError::MissingStatusLine);
        }

        let mut status_line_iter = status_line.trim().split(" ");

        let method = status_line_iter
            .next()
            .ok_or(HttpRequestParseError::MissingMethod)?;
        let path = status_line_iter
            .next()
            .ok_or(HttpRequestParseError::MissingPath)?;

        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            let len = reader
                .read_line(&mut line)
                .map_err(|error| HttpRequestParseError::StreamError(error))?;

            let line = line.trim();

            if line.is_empty() || len == 0 {
                break;
            }

            let mut header_it = line.split(":");
            let name = header_it
                .next()
                .ok_or(HttpRequestParseError::MissingHeaderName)?;
            let value = header_it
                .next()
                .ok_or(HttpRequestParseError::MissingHeaderValue(name.to_owned()))?;

            headers.insert(name.to_owned(), value.to_owned());
        }

        Ok(HttpRequest::new(
            method.to_owned(),
            path.to_owned(),
            headers,
            reader,
        ))
    }
}

pub trait HttpInnerStream {
    fn unpack_stream(self) -> BufReader<TcpStream>;
}

impl HttpInnerStream for HttpRequest {
    fn unpack_stream(self) -> BufReader<TcpStream> {
        self.body_stream
    }
}

#[derive(Debug)]
pub enum HttpRequestParseError {
    StreamError(std::io::Error),
    MissingStatusLine,
    MissingMethod,
    MissingPath,
    MissingHeaderName,
    MissingHeaderValue(String),
}

impl Display for HttpRequestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::StreamError(error) => format!("Stream error: {}.", error),
            Self::MissingStatusLine => String::from("Missing status line."),
            Self::MissingMethod => String::from("Missing method."),
            Self::MissingPath => String::from("Missing path."),
            Self::MissingHeaderName => String::from("Missing header name. Maybe an empy header?"),
            Self::MissingHeaderValue(header) => format!("Missing header value for {header}"),
        };

        f.write_str(&format!("Error wile parsing HttpRequest: {}", msg))
    }
}

impl Error for HttpRequestParseError {}
