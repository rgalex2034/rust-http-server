use std::{
    collections::HashMap,
    io::Write,
    net::TcpListener,
    sync::{Arc, Mutex},
};

use crate::thread_pool::ThreadPool;
use crate::{HttpRequest, HttpResponse};

pub struct HttpServer {
    thread_pool: ThreadPool,
    tcp_listener: TcpListener,
    handlers: Arc<Mutex<HashMap<(String, String), Arc<RequestHandler>>>>,
}

type RequestHandler = dyn (Fn(HttpRequest) -> HttpResponse) + Send + Sync + 'static;

impl HttpServer {
    pub fn new(bind_address: &str, pool_size: usize) -> HttpServer {
        let thread_pool = ThreadPool::new(pool_size);
        let tcp_listener = TcpListener::bind(bind_address)
            .expect("Unable to bind TcpListener while creating HttpServer");
        HttpServer {
            thread_pool,
            tcp_listener,
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get<F>(&mut self, path: &str, f: F)
    where
        F: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        self.handlers
            .lock()
            .unwrap()
            .insert(("GET".to_owned(), path.to_owned()), f);
    }

    pub fn run(&self) {
        println!(
            "Listening on http://{}",
            self.tcp_listener.local_addr().unwrap()
        );
        for stream in self.tcp_listener.incoming() {
            let mut stream =
                stream.expect("TcpListener dropped. Can not get incoming connections.");
            let handlers = Arc::clone(&self.handlers);
            let result = self.thread_pool.execute(move || {
                let request: HttpRequest = match (&stream).try_into() {
                    Ok(request) => request,
                    Err(error) => {
                        eprintln!("Invalid HTTP request:\n{error}");
                        return;
                    }
                };

                println!(
                    "Request received: {} {} from {}",
                    request.method(),
                    request.path(),
                    stream.peer_addr().unwrap().to_string()
                );

                let handlers = handlers.lock().unwrap();

                let handler =
                    handlers.get(&(request.method().to_owned(), request.path().to_owned()));

                let response = if let Some(f) = handler {
                    // Clone the function. It's on an Arc, so it can be shared.
                    // Then drop `handlers` to release the lock and allow other workers
                    // to access the closure map and process requests in parellel.
                    let f = Arc::clone(f);
                    drop(handlers);
                    f(request)
                } else {
                    HttpResponse::http_404()
                };

                let _ = stream
                    .write_all(response.to_string().as_bytes())
                    .inspect_err(|error| {
                        eprintln!("HttpServer: Error while writing response: {}", error)
                    });
            });

            let _ = result.inspect_err(|error| eprintln!("{error}"));
        }
    }
}
