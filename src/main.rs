use std::{thread, time::Duration};

use async_http::{self, HttpResponse, HttpServer};

fn main() {
    let mut server = HttpServer::new("127.0.0.1:7878", 5);

    server.get("/", |_request| {
        let mut response = HttpResponse::new(200);

        response.set_body("text/html", "<h1>Hello world</h1>");

        thread::sleep(Duration::from_secs(3));

        response
    });

    server.run();
}
