import React, { useEffect, useRef, useState } from 'react';
import Layer from './Layer';
import LayerDetails from './LayerDetails';
import './index.css';

async function registerUser() {
  console.log(`Using domain ${window.location.origin}`);
  const response = await fetch(`${window.location.origin}/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ user_id: 1 })
  });

  if (!response.ok) {
    throw new Error('Failed to register user');
  }

  const data = await response.json();
  return data; // Assuming the response contains a field 'websocket_url'
}

function connectWebSocket(socketInfo, setMessage, setSocket, setConnectionStatus) {
  let url = `ws://${window.location.hostname}:${socketInfo.port}/ws/${socketInfo.uuid}`;
  const socket = new WebSocket(url);

  socket.onopen = function () {
    console.log('WebSocket connection established');
    setConnectionStatus('connected');
  };

  socket.onmessage = function (event) {
    setMessage(event.data);
  };

  socket.onclose = function () {
    console.log('WebSocket connection closed');
    setConnectionStatus('disconnected');
  };

  socket.onerror = function (error) {
    console.error('WebSocket error:', error);
    setConnectionStatus('error');
  };

  setSocket(socket);
  return socket;
}

async function init(setMessage, setSocket, setConnectionStatus) {
  console.log("Debugger client init");

  try {
    setConnectionStatus('connecting');
    const websocket = await registerUser();
    return connectWebSocket(websocket, setMessage, setSocket, setConnectionStatus);
  } catch (error) {
    console.error('Error during initialization:', error);
    setConnectionStatus('error');
    return null;
  }
}


function App() {
  const [message, setMessage] = useState('');
  const [socket, setSocket] = useState(null);
  const [selectedLayer, setSelectedLayer] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('idle');
  const [theme, setTheme] = useState('light');
  const [treeWidth, setTreeWidth] = useState(360);
  const resizing = useRef(null);
  const [search, setSearch] = useState('');

  useEffect(() => {
    let activeSocket = null;

    init(setMessage, setSocket, setConnectionStatus).then((socket) => {
      activeSocket = socket;
    });

    return () => {
      if (activeSocket) {
        activeSocket.close();
      }
    };
  }, []);

  useEffect(() => {
    document.body.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    const onMove = (ev) => {
      if (!resizing.current) return;
      const delta = ev.clientX - resizing.current.startX;
      const nextWidth = Math.min(Math.max(resizing.current.startWidth + delta, 240), 700);
      setTreeWidth(nextWidth);
      ev.preventDefault();
    };

    const onUp = () => {
      resizing.current = null;
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
  }, []);

  let layers = null;
  let root = null;
  let root_id = null;
  if (message !== '') {
    layers = JSON.parse(message);
    root_id = layers[0];
    layers = layers[1];
    root = layers[root_id];
  }

  const matchesSearch = (layerEntry) => {
    const term = search.trim();
    if (!term) return true;

    const [id, attrs] = layerEntry || [];
    const normalized = term.toLowerCase();

    if (normalized.startsWith('id:')) {
      const idQuery = normalized.slice(3).trim();
      const parsed = Number(idQuery);
      if (!Number.isNaN(parsed)) {
        return id === parsed;
      }
    }

    const name = attrs?.key ?? '';
    return name.toLowerCase().includes(normalized);
  };

  const filteredLayers = () => {
    if (!layers) return null;
    const copy = { ...layers };
    // Hide children entirely if they don't match and none of their descendants match.
    const cache = {};
    const hasMatch = (id) => {
      if (cache[id] !== undefined) return cache[id];
      const entry = copy[id];
      if (!entry) return (cache[id] = false);
      if (matchesSearch(entry)) return (cache[id] = true);
      const kids = entry[2] || [];
      for (const cid of kids) {
        if (hasMatch(cid)) return (cache[id] = true);
      }
      return (cache[id] = false);
    };
    const filtered = {};
    Object.keys(copy).forEach((k) => {
      const id = Number(k);
      if (hasMatch(id)) {
        const entry = copy[id];
        filtered[id] = [entry[0], entry[1], (entry[2] || []).filter(hasMatch), entry[3]];
      }
    });
    return filtered;
  };

  const sendMessage = (msg) => {
    let stringy_msg = JSON.stringify(msg);
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(stringy_msg);
      console.log('Message sent:', stringy_msg);
    } else {
      console.error('WebSocket is not open');
    }
  };

  const statusLabel = {
    idle: 'Idle',
    connecting: 'Connecting…',
    connected: 'Connected',
    disconnected: 'Disconnected',
    error: 'Error',
  }[connectionStatus];

  return (
    <div className="App">
      <header className="toolbar">
        <div className="brand">
          <span className="brand-accent"></span>
          Layers Inspector
        </div>
        <div className="toolbar-actions">
          <button
            className="icon-toggle"
            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
            title="Toggle theme"
            aria-label="Toggle theme"
          >
            {theme === 'dark' ? '☀️' : '🌙'}
          </button>
          <div className={`status-pill ${connectionStatus}`}>
            <span className="status-dot" />
            {statusLabel}
          </div>
        </div>
      </header>

      <div className="pane-layout" style={{ gridTemplateColumns: `${treeWidth}px 8px 1fr` }}>
        <section className="panel tree-panel">
          <div className="panel-header">
            <div>
              <div className="panel-title">Scene graph</div>
            </div>
            <div className="pill">Tree</div>
          </div>
          <div className="panel-toolbar">
            <div className="search-wrapper">
                <input
                  className="search-input"
                  type="text"
                  placeholder="Search layers or id:33"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
                {search && (
                  <button
                    className="clear-btn"
                    onClick={() => setSearch('')}
                    aria-label="Clear search"
                  >
                    ×
                  </button>
                )}
              </div>
            </div>
            <div className="panel-body scroll-body">
              {root && (() => {
                const filtered = filteredLayers();
                const layersForTree = filtered || layers;
                const rootEntry = layersForTree ? layersForTree[root_id] : null;
                if (!rootEntry) {
                  return <div className="empty-state">No layers match your search.</div>;
                }
                return (
                  <Layer
                    key={root_id}
                    layer={rootEntry}
                    layers={layersForTree}
                    sendMessage={sendMessage}
                    setSelectedLayer={setSelectedLayer}
                    selectedLayer={selectedLayer}
                  />
                );
              })()}
            </div>
        </section>

        <div
          className="resize-handle"
          onMouseDown={(ev) => {
            resizing.current = { startX: ev.clientX, startWidth: treeWidth };
          }}
        />

        <section className="panel details-panel">
          <div className="panel-header">
            <div>
              <div className="panel-title">Details</div>
            </div>
            {selectedLayer && <div className="pill muted">#{selectedLayer[0]}</div>}
          </div>
          <div className="panel-body scroll-body">
            {selectedLayer ? (
              <LayerDetails layer={selectedLayer} rootLayer={root} layers={layers} rootId={root_id} />
            ) : (
              <div className="empty-state">Select a layer from the tree.</div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

export default App;
