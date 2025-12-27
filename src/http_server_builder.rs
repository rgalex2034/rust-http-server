use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    HttpRequest, HttpResponse, HttpServer, http_dispatcher::HttpDispatcher, thread_pool::ThreadPool,
};

pub struct HttpServerBuilder {
    bind_address: String,
    thread_pool_size: usize,
    handlers: HashMap<(String, String), Arc<RequestHandler>>,
}

type RequestHandler = dyn (Fn(&HttpRequest) -> HttpResponse) + Send + Sync + 'static;

impl HttpServerBuilder {
    pub fn new(bind_address: &str, thread_pool_size: usize) -> HttpServerBuilder {
        HttpServerBuilder {
            bind_address: bind_address.to_owned(),
            thread_pool_size,
            handlers: HashMap::new(),
        }
    }

    pub fn get<F>(&mut self, path: &str, f: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route("GET", path, f);
    }

    pub fn route<F>(&mut self, method: &str, path: &str, f: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let key = (method.to_owned(), path.to_owned());
        self.handlers.insert(key, Arc::new(f));
    }

    pub fn build(self) -> HttpServer {
        let http_dispatcher =
            HttpDispatcher::new(self.handlers, ThreadPool::new(self.thread_pool_size));
        HttpServer::new(&self.bind_address, http_dispatcher)
    }
}
