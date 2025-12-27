use std::{collections::HashMap, fmt::Display};

pub struct HttpResponse {
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        HttpResponse {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn http_404() -> Self {
        HttpResponse::new(HttpStatus::NotFound)
    }

    pub fn set_status(&mut self, status: HttpStatus) {
        self.status = status;
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_owned(), value.to_owned());
    }

    pub fn set_body(&mut self, content_type: &str, content: &str) {
        self.set_header("Content-Type", content_type);
        self.set_header("Content-Length", &content_type.len().to_string());
        self.body = Some(String::from(content));
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_text = format!("HTTP/1.1 {}", self.status);
        let headers_text = self
            .headers
            .iter()
            .map(|header| format!("{}: {}", header.0, header.1))
            .collect::<Vec<_>>()
            .join("\r\n");

        write!(f, "{}\r\n", status_text)
            .and(write!(f, "{}\r\n\r\n", headers_text))
            .and_then(|_| {
                if let Some(body) = &self.body {
                    write!(f, "{}", body)
                } else {
                    Ok(())
                }
            })
    }
}

pub enum HttpStatus {
    Ok,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl From<&HttpStatus> for u32 {
    fn from(value: &HttpStatus) -> Self {
        match value {
            HttpStatus::Ok => 200,
            HttpStatus::BadRequest => 400,
            HttpStatus::NotFound => 404,
            HttpStatus::InternalServerError => 500,
        }
    }
}

impl From<&HttpStatus> for &str {
    fn from(value: &HttpStatus) -> Self {
        match value {
            HttpStatus::Ok => "OK",
            HttpStatus::BadRequest => "Bad request",
            HttpStatus::NotFound => "Not found",
            HttpStatus::InternalServerError => "Internal server error",
        }
    }
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_code: u32 = self.into();
        let status_msg: &str = self.into();

        write!(f, "{} {}", status_code, status_msg)
    }
}
