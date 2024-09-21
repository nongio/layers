use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    filters::ws::{Message, WebSocket},
    reply::{json, Reply},
};

use super::{DebuggerClient, DebuggerEvent, RegisterRequest, RegisterResponse, Result};
use crate::{with_client, with_server, DebugServerError, CLIENT};

pub async fn register_handler(body: RegisterRequest) -> Result<impl Reply> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();
    register_client(uuid.clone(), user_id).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
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
