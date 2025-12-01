# Layers Inspector

The Layers Inspector is a lightweight debugger that exposes a live view of the engine scene graph in the browser. It ships with a web client for navigation and a small Warp server that bridges the client to the engine at runtime.

## What you get
- Live scene graph tree with search that hides non‑matching branches while you type.
- Click a layer to inspect its attributes; use the viewport highlight toggle to ask the engine to outline the node.
- Resizeable tree/details panes with light/dark themes for quick checks during development.

## How it works
- The `layers-debug-server` crate hosts HTTP and WebSocket endpoints on port `8000` and serves the built React client from `packages/debugger_server/client/build`.
- The engine implements the `layers_debug_server::DebugServer` trait (behind the `debugger` feature) and responds to client commands such as `["highlight", <node_id>]` and `["unhighlight", <node_id>]`.
- The client registers via `POST /register`, then opens a WebSocket at `/ws/{uuid}` to receive serialized layer trees and send commands back.
- Layer tree snapshots include node ids and keys, which the client renders as the searchable tree on the left.
  - Quick filter syntax: type `id:33` to jump directly to node `33` by id; any other text matches layer keys.

## Running it locally
1. Build your app with the debugger feature so the server is available, for example:
   ```bash
   cargo run --features "default,debugger" -p hello-views
   ```
2. Call `engine.start_debugger()` once after creating the engine to spawn the Warp server.
3. Open `http://localhost:8000/client/index.html` in your browser. The left column shows the current tree with a search input; the right column displays details for the selected layer.
4. Toggle the dot icon on any row to highlight or clear the node in the viewport; the client sends `highlight`/`unhighlight` messages that the engine handles via `DebugServer`.

## Extending the inspector
- The web client lives in `packages/debugger_server/client` (React). Update the UI, run `npm run build`, and rebuild your Rust target to ship the new assets.
- Server behavior (routing, registration, message handling) is defined in `packages/debugger_server/src`. Add new commands or topics there and implement them on the engine side.
