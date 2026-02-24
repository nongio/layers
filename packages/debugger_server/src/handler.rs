use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    filters::ws::{Message, WebSocket},
    reply::{json, Reply},
};

use std::collections::HashMap;

use super::{DebuggerClient, DebuggerEvent, RegisterRequest, RegisterResponse, Result};
use crate::{
    with_client, with_server, DebugServerError, CLIENT, DEBUGGER_PORT, LAST_SCENE_SNAPSHOT,
};

pub async fn register_handler(body: RegisterRequest) -> Result<impl Reply> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();

    register_client(uuid.clone(), user_id).await;
    let port = DEBUGGER_PORT.get().copied().unwrap_or(8000);
    Ok(json(&RegisterResponse { uuid, port }))
}

pub async fn register_client(_id: String, user_id: usize) {
    let _ = CLIENT.set(RwLock::new(Some(DebuggerClient {
        user_id,
        topics: vec![],
        sender: None,
    })));
}

pub fn health_handler() -> impl FutureExt<Output = Result<impl Reply>> {
    futures::future::ready(Ok(warp::http::StatusCode::OK))
}

pub async fn unregister_handler(_id: String) -> Result<impl Reply> {
    let _ = CLIENT.set(RwLock::new(None));

    Ok(warp::http::StatusCode::OK)
}

pub async fn scene_handler(node_id: usize) -> Result<warp::reply::Response> {
    if let Some(snapshot) = LAST_SCENE_SNAPSHOT.get() {
        let snapshot = snapshot.read().await;
        if let Some(snapshot) = snapshot.as_ref() {
            let filtered = if node_id == 0 {
                Some(snapshot.clone())
            } else {
                filter_subtree(snapshot, node_id)
            };

            if let Some(result) = filtered {
                return Ok(warp::reply::with_header(
                    result,
                    "content-type",
                    "application/json; charset=utf-8",
                )
                .into_response());
            } else {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({"error": "node_id not found"})),
                    warp::http::StatusCode::NOT_FOUND,
                )
                .into_response());
            }
        }
    }

    Ok(warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT).into_response())
}

fn filter_subtree(snapshot: &str, root_node_id: usize) -> Option<String> {
    // Parse the snapshot: (root_id, HashMap<usize, (id, layer, children, node_id)>)
    #[allow(clippy::type_complexity)]
    let parsed: (
        usize,
        HashMap<usize, (usize, serde_json::Value, Vec<usize>, serde_json::Value)>,
    ) = serde_json::from_str(snapshot).ok()?;

    let (_, all_nodes) = parsed;

    // Check if the requested node exists
    if !all_nodes.contains_key(&root_node_id) {
        return None;
    }

    // Collect all descendants of root_node_id using BFS/DFS
    let mut subtree_nodes = HashMap::new();
    let mut to_visit = vec![root_node_id];

    while let Some(node_id) = to_visit.pop() {
        if let Some(node_data) = all_nodes.get(&node_id) {
            let children = &node_data.2;
            to_visit.extend(children);
            subtree_nodes.insert(node_id, node_data.clone());
        }
    }

    // Serialize the subtree with the new root
    let result = (root_node_id, subtree_nodes);
    serde_json::to_string(&result).ok()
}

pub async fn publish_handler(body: DebuggerEvent) -> Result<impl Reply> {
    with_client(|client| {
        if client.topics.contains(&body.topic) {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        }
    })
    .await;

    Ok(warp::http::StatusCode::OK)
}

pub async fn client_connection(ws: WebSocket, _id: String) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    if let Some(client) = CLIENT.get() {
        let mut client = client.write().await;
        client.replace(DebuggerClient {
            user_id: 0,
            topics: vec![],
            sender: Some(client_sender),
        });
    }
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    // Process incoming WebSocket messages
    while let Some(result) = client_ws_rcv.next().await {
        let result = result
            .map(|m| m.to_str().unwrap().to_string())
            .map_err(|_e| DebugServerError {});

        with_server(|server| {
            server.handle_message(result);
        })
        .await;
    }
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| client_connection(socket, id)))
}
