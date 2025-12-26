use std::{collections::HashMap, fmt::Display};

pub struct HttpResponse {
    status: u32,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpResponse {
    pub fn new(status: u32) -> Self {
        HttpResponse {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn http_404() -> Self {
        HttpResponse::new(404)
    }

    pub fn set_status(&mut self, status: u32) {
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

    fn status_to_str(&self) -> String {
        let status_text = match self.status {
            200 => "OK",
            400 => "Bad request",
            404 => "Not found",
            500 => "Internal server error",
            _ => "Unknown",
        };

        status_text.to_owned()
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_text = format!("HTTP/1.1 {} {}", self.status, self.status_to_str());
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
