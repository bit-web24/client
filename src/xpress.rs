use std::future::Future;
use std::sync::Arc;
use tokio::io::Result;
use tokio::net::{TcpListener, ToSocketAddrs};
mod handler;
pub mod router;

use handler::RequestHandler;
use router::request::Request;
use router::response::Response;
use router::{Route, Router};
use tokio::sync::Mutex;
pub struct App {
    name: String,
    routes: Arc<Mutex<Router>>,
}

impl App
// where
//     F: Fn(Request, Response) -> Fut + Send + 'static + Clone + Sync,
//     Fut: Future<Output = Result<()>> + Send + 'static,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            routes: Arc::new(Mutex::new(Router::new())),
        }
    }

    pub async fn get<F>(&mut self, path: &str, callback: F) -> Result<()>
    where
        F: Fn(
                Request,
                Response,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let route = Route::new(path, router::Method::Get, callback);

        let mut routes = self.routes.lock().await;

        routes.add(route);
        Ok(())
    }

    // pub fn post(&self, route: &str, callback: F) -> Result<()> {}
    // pub fn put(&self, route: &str, callback: F) -> Result<()> {}
    // pub fn delete(&self, route: &str, callback: F) -> Result<()> {}

    pub async fn listen<A, B>(&mut self, addr: A, callback: B) -> Result<()>
    where
        A: ToSocketAddrs,
        B: Fn() -> Result<()>,
    {
        let listener = TcpListener::bind(addr).await?;
        callback()?;

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("CONNECTED: {addr}");

            let mut handler = RequestHandler::from(Arc::clone(&self.routes));

            tokio::spawn(async move {
                handler.handle(socket).await.expect("Error: handler error");
            })
            .await?;
        }
    }
}
