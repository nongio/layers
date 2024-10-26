mod handler;

use std::sync::{Arc, OnceLock};

use static_dir::static_dir;
use tokio::sync::{mpsc, RwLock};
use warp::{filters::ws::Message, reject::Rejection, Filter};

#[derive(Clone)]
pub struct DebuggerClient {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct RegisterRequest {
    user_id: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RegisterResponse {
    uuid: String,
    port: u16,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct DebuggerEvent {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

type Result<T> = std::result::Result<T, Rejection>;
type Server = RwLock<Option<Arc<dyn DebugServer>>>;
type Client = RwLock<Option<DebuggerClient>>;

pub(crate) static CLIENT: OnceLock<Client> = OnceLock::new();
pub(crate) static SERVER: OnceLock<Server> = OnceLock::new();

async fn with_client<F: FnOnce(DebuggerClient)>(f: F) {
    if let Some(client) = CLIENT.get() {
        let client = client.read().await;
        if let Some(client) = client.clone() {
            f(client)
        }
    }
}

async fn with_server<F: FnOnce(Arc<dyn DebugServer>)>(f: F) {
    if let Some(server) = SERVER.get() {
        let server = server.read().await;
        if let Some(server) = server.clone() {
            f(server)
        }
    }
}
pub fn start_debugger_server(debug_server: Arc<dyn DebugServer>) {
    tokio::spawn(async move { start_debugger(debug_server).await });
}
async fn start_debugger(debug_server: Arc<dyn DebugServer>) {
    let _ = SERVER.set(RwLock::new(Some(debug_server)));
    let _ = CLIENT.set(RwLock::new(None)); // Initialize CLIENT as needed

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        // .and(with_clients())
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            // .and(with_clients())
            .and_then(handler::unregister_handler));

    let publish = warp::path!("publish")
        .and(warp::body::json())
        // .and(with_clients())
        .and_then(handler::publish_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        // .and(with_clients())
        .and_then(handler::ws_handler);

    let client_files = warp::path("client").and(static_dir!("client/build/"));
    let cors = warp::cors()
        .allow_origins(vec![
            "http://localhost:3000",
            "http://localhost:8000",
            "http://192.168.122.246:8000",
        ])
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "strict-origin-when-cross-origin",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "user-agent",
            "content-type",
        ])
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"]);
    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .or(client_files)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

pub struct DebugServerError;
pub trait DebugServer: Send + Sync + 'static {
    fn handle_message(&self, message: std::result::Result<String, DebugServerError>);
}

pub fn send_debugger_message(message: String) {
    tokio::spawn(async move {
        with_client(|client| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message.clone())));
            }
        })
        .await;
    });
}
