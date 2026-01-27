mod handler;

use std::{
    env,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

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
pub(crate) static LAST_SCENE_SNAPSHOT: OnceLock<RwLock<Option<String>>> = OnceLock::new();
pub(crate) static DEBUGGER_PORT: OnceLock<u16> = OnceLock::new();

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
    let _ = LAST_SCENE_SNAPSHOT.set(RwLock::new(None));

    let desired_port = env::var("LAYERS_DEBUGGER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .filter(|p| *p != 0)
        .unwrap_or(8000);

    let _ = DEBUGGER_PORT.set(desired_port);

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let scene_route = warp::path("scene")
        .and(warp::get())
        .and(
            warp::path::param::<usize>()
                .or(warp::path::end().map(|| 0))
                .unify(),
        )
        .and_then(handler::scene_handler);

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
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("client/build");
    let client_files = warp::path("client").and(warp::fs::dir(package_dir));

    let allowed_origins: Vec<String> = vec![
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
        format!("http://localhost:{}", desired_port),
        format!("http://127.0.0.1:{}", desired_port),
    ];

    let cors = warp::cors()
        .allow_origins(
            allowed_origins
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )
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
        .or(scene_route)
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .or(client_files)
        .with(cors);

    let bind_addr = ([0, 0, 0, 0], desired_port);

    // Prefer the configured port; if it's taken, fall back to any free port.
    if let Ok((bound_addr, server_fut)) = warp::serve(routes.clone()).try_bind_ephemeral(bind_addr)
    {
        let _ = DEBUGGER_PORT.set(bound_addr.port());
        server_fut.await;
        return;
    }

    eprintln!(
        "debugger server: failed to bind to port {} — falling back to an ephemeral port",
        desired_port
    );
    let (bound_addr, server_fut) = warp::serve(routes).bind_ephemeral(([0, 0, 0, 0], 0));
    let _ = DEBUGGER_PORT.set(bound_addr.port());
    server_fut.await;
}

pub struct DebugServerError;
pub trait DebugServer: Send + Sync + 'static {
    fn handle_message(&self, message: std::result::Result<String, DebugServerError>);
}

pub fn send_debugger_message(message: String) {
    tokio::spawn(async move {
        if let Some(snapshot) = LAST_SCENE_SNAPSHOT.get() {
            let mut snapshot = snapshot.write().await;
            snapshot.replace(message.clone());
        }
        with_client(|client| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message.clone())));
            }
        })
        .await;
    });
}
