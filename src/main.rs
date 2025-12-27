use std::{thread, time::Duration};

use async_http::{self, HttpResponse, HttpServerBuilder, HttpStatus};

fn main() {
    let mut builder = HttpServerBuilder::new("127.0.0.1:7878", 5);

    builder.get("/", |_request| {
        let mut response = HttpResponse::new(HttpStatus::Ok);

        response.set_body("text/html", "<h1>Hello world</h1>");

        thread::sleep(Duration::from_secs(3));

        response
    });

    let server = builder.build();

    server.run();
}
