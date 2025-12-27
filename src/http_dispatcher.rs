use crate::HttpRequest;
use crate::HttpResponse;
use crate::http_request::HttpInnerStream;
use crate::thread_pool::ThreadPool;
use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::{collections::HashMap, net::TcpStream};

pub struct HttpDispatcher {
    sender: mpsc::Sender<TcpStream>,
    join_handle: Option<thread::JoinHandle<()>>,
}

type RequestHandler = dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static;

impl HttpDispatcher {
    pub fn new(
        handlers: HashMap<(String, String), Arc<RequestHandler>>,
        thread_pool: ThreadPool,
    ) -> HttpDispatcher {
        let (sender, join_handle) = Self::spawn_dispatcher(thread_pool, handlers);

        HttpDispatcher {
            sender,
            join_handle: Some(join_handle),
        }
    }

    fn spawn_dispatcher(
        thread_pool: ThreadPool,
        handlers: HashMap<(String, String), Arc<RequestHandler>>,
    ) -> (mpsc::Sender<TcpStream>, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            let http_404: Arc<RequestHandler> =
                Arc::new(|_: &HttpRequest| HttpResponse::http_404());

            loop {
                let tcp_stream: TcpStream = receiver
                    .recv()
                    .expect("Dispatcher: sender not working. Stopping receiving request.");

                let request: HttpRequest = match tcp_stream.try_into() {
                    Ok(request) => request,
                    Err(error) => {
                        eprintln!("Invalid HTTP request:\n{error}");
                        continue;
                    }
                };

                let handler = Arc::clone(
                    handlers
                        .get(&(request.method().to_owned(), request.path().to_owned()))
                        .unwrap_or(&http_404),
                );

                thread_pool
                    .execute(move || {
                        let response = handler(&request);
                        let _ = request
                            .unpack_stream()
                            .into_inner()
                            .write_all(response.to_string().as_bytes())
                            .inspect_err(|error| {
                                eprintln!("HttpServer: Error while writing response: {}", error)
                            });
                    })
                    .expect("Dispatcher: Can not send work to thread pool.");
            }
        });

        (sender, join_handle)
    }

    pub fn dispatch(&self, tcp_stream: TcpStream) -> Result<(), mpsc::SendError<TcpStream>> {
        self.sender.send(tcp_stream)
    }
}

impl Drop for HttpDispatcher {
    fn drop(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join().inspect_err(|_| {
                eprintln!("Dispatcher: Error while cleaning up dispatcher thread.");
            });
        };
    }
}
