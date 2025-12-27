use std::net::TcpListener;

use crate::http_dispatcher::HttpDispatcher;

pub struct HttpServer {
    tcp_listener: TcpListener,
    http_dispatcher: HttpDispatcher,
}

impl HttpServer {
    pub fn new(bind_address: &str, http_dispatcher: HttpDispatcher) -> HttpServer {
        let tcp_listener = TcpListener::bind(bind_address)
            .expect("Unable to bind TcpListener while creating HttpServer");
        HttpServer {
            tcp_listener,
            http_dispatcher,
        }
    }

    pub fn run(&self) {
        println!(
            "Listening on http://{}",
            self.tcp_listener.local_addr().unwrap()
        );
        for stream in self.tcp_listener.incoming() {
            let stream = stream.expect("TcpListener dropped. Can not get incoming connections.");

            let result = self.http_dispatcher.dispatch(stream);

            let _ = result.inspect_err(|error| eprintln!("{error}"));
        }
    }
}
