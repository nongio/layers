#[cfg(feature = "debugger")]
use layers_debug_server::DebugServerError;

#[cfg(feature = "debugger")]
impl layers_debug_server::DebugServer for crate::engine::Engine {
    fn handle_message(&self, result: std::result::Result<String, DebugServerError>) {
        match result {
            Ok(msg) => {
                if let Ok((command, node_id)) =
                    serde_json::from_str::<(String, indextree::NodeId)>(msg.as_str())
                {
                    match command.as_str() {
                        "highlight" => {
                            self.scene.with_arena(|arena| {
                                let node = arena.get(node_id).unwrap();
                                let scene_node: &crate::engine::node::SceneNode = node.get();
                                scene_node.set_debug_info(true);
                            });
                        }
                        "unhighlight" => {
                            self.scene.with_arena(|arena| {
                                let node = arena.get(node_id).unwrap();
                                let scene_node: &crate::engine::node::SceneNode = node.get();
                                scene_node.set_debug_info(false);
                            });
                        }

                        _ => {
                            println!("Unknown command: {}", command);
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("error receiving websocket msg");
            }
        }
    }
}

#[cfg(feature = "debugger")]
impl crate::engine::Engine {
    /// Start the debugger server
    ///
    /// Can be accessed at `http://localhost:8000/client/index.html`
    pub fn start_debugger(&self) {
        layers_debug_server::start_debugger_server(self.get_arc_ref());
    }
}
