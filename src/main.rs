use async_http::{self, HttpResponse, HttpServerBuilder, HttpStatus};

fn main() {
    let mut builder = HttpServerBuilder::new("127.0.0.1:7878", num_cpus::get() * 25);

    builder.get("/", |_request| {
        let mut response = HttpResponse::new(HttpStatus::Ok);

        response.set_body("text/html", "<h1>Hello world</h1>");

        response
    });

    builder.get("/statuses", |_request| {
        let mut response = HttpResponse::new(HttpStatus::Ok);

        let mut body = String::from("<h1>HTTP Status Codes</h1>");

        for num in 100..=599 {
            let status: HttpStatus = match num.try_into() {
                Ok(status) => status,
                _ => continue,
            };

            body.push_str(&format!("{}<br/>", status));
        }

        response.set_body("text/html", &body);

        response
    });

    let server = builder.build();

    server.run();
}
