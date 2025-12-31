import React, { useEffect, useMemo, useRef, useState } from 'react';
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
  const [selectedLayerId, setSelectedLayerId] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('idle');
  const [theme, setTheme] = useState('light');
  const defaultLayoutWidths = useMemo(() => {
    const width = typeof window !== 'undefined' ? window.innerWidth : 1200;
    return {
      tree: Math.round(width * 0.25),
      right: Math.round(width * 0.25),
    };
  }, []);
  const [treeWidth, setTreeWidth] = useState(defaultLayoutWidths.tree);
  const [rightWidth, setRightWidth] = useState(defaultLayoutWidths.right);
  const resizing = useRef(null);
  const [search, setSearch] = useState('');
  const [expanded, setExpanded] = useState({});
  const treeBodyRef = useRef(null);
  const keyboardScrollRef = useRef(false);

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
      const totalWidth = window.innerWidth;
      const handleSize = 16;
      const minPanel = 240;
      const minMiddle = 320;
      const { target, startX, startWidth } = resizing.current;
      const delta = ev.clientX - startX;
      if (target === 'tree') {
        const max = totalWidth - minMiddle - rightWidth - handleSize;
        const nextWidth = Math.min(Math.max(startWidth + delta, minPanel), Math.max(max, minPanel));
        setTreeWidth(nextWidth);
      } else if (target === 'right') {
        const max = totalWidth - minMiddle - treeWidth - handleSize;
        const nextWidth = Math.min(Math.max(startWidth - delta, minPanel), Math.max(max, minPanel));
        setRightWidth(nextWidth);
      }
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
  const selectedLayer = selectedLayerId !== null ? layers?.[selectedLayerId] : null;

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

  const isExpanded = (id) => {
    if (expanded[id] === undefined) {
      const depth = depthMap[id];
      if (depth === undefined) return id === root_id;
      return depth <= 1;
    }
    return expanded[id];
  };

  const toggleExpanded = (id) => {
    setExpanded((prev) => {
      const current = prev[id] === undefined ? true : prev[id];
      return { ...prev, [id]: !current };
    });
  };

  const setExpandedFor = (id, value) => {
    setExpanded((prev) => {
      if (prev[id] === value) return prev;
      return { ...prev, [id]: value };
    });
  };

  const filtered = layers ? filteredLayers() : null;
  const layersForTree = filtered || layers;
  const rootEntry = layersForTree && root_id !== null ? layersForTree[root_id] : null;

  const parentMap = useMemo(() => {
    if (!layersForTree) return {};
    const parents = {};
    Object.values(layersForTree).forEach((entry) => {
      const kids = entry[2] || [];
      kids.forEach((cid) => {
        parents[cid] = entry[0];
      });
    });
    return parents;
  }, [layersForTree]);

  const depthMap = useMemo(() => {
    if (!layersForTree || root_id === null) return {};
    const depths = {};
    const walk = (id, depth) => {
      depths[id] = depth;
      const entry = layersForTree[id];
      if (!entry) return;
      const kids = entry[2] || [];
      kids.forEach((cid) => walk(cid, depth + 1));
    };
    walk(root_id, 0);
    return depths;
  }, [layersForTree, root_id]);

  const visibleIds = useMemo(() => {
    if (!layersForTree || root_id === null) return [];
    const list = [];
    const walk = (id) => {
      const entry = layersForTree[id];
      if (!entry) return;
      list.push(id);
      const kids = entry[2] || [];
      if (isExpanded(id)) {
        kids.forEach(walk);
      }
    };
    walk(root_id);
    return list;
  }, [layersForTree, root_id, expanded]);

  useEffect(() => {
    if (!layersForTree || root_id === null) return;

    const onKeyDown = (ev) => {
      const tag = ev.target?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || ev.target?.isContentEditable) {
        return;
      }

      if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(ev.key)) {
        return;
      }

      ev.preventDefault();
      if (visibleIds.length === 0) return;

      let currentId = selectedLayerId ?? root_id;
      if (!visibleIds.includes(currentId)) {
        currentId = root_id;
      }

      const index = visibleIds.indexOf(currentId);
      const currentEntry = layersForTree[currentId];
      const children = currentEntry?.[2] || [];

      if (ev.key === 'ArrowDown') {
        if (index < visibleIds.length - 1) {
          keyboardScrollRef.current = true;
          setSelectedLayerId(visibleIds[index + 1]);
        }
        return;
      }

      if (ev.key === 'ArrowUp') {
        if (index > 0) {
          keyboardScrollRef.current = true;
          setSelectedLayerId(visibleIds[index - 1]);
        }
        return;
      }

      if (ev.key === 'ArrowRight') {
        if (children.length > 0) {
          if (!isExpanded(currentId)) {
            keyboardScrollRef.current = true;
            setExpandedFor(currentId, true);
          } else {
            keyboardScrollRef.current = true;
            setSelectedLayerId(children[0]);
          }
        }
        return;
      }

      if (ev.key === 'ArrowLeft') {
        if (children.length > 0 && isExpanded(currentId)) {
          keyboardScrollRef.current = true;
          setExpandedFor(currentId, false);
          return;
        }

        const parentId = parentMap[currentId];
        if (parentId !== undefined) {
          keyboardScrollRef.current = true;
          setSelectedLayerId(parentId);
        }
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [layersForTree, root_id, visibleIds, selectedLayerId, parentMap, expanded]);

  useEffect(() => {
    if (selectedLayerId === null) return;
    if (!keyboardScrollRef.current) return;
    keyboardScrollRef.current = false;
    const container = treeBodyRef.current;
    if (!container) return;
    const row = container.querySelector(`[data-layer-row-id="${selectedLayerId}"]`);
    if (!row) return;
    const containerRect = container.getBoundingClientRect();
    const rowRect = row.getBoundingClientRect();
    const deltaTop = rowRect.top - containerRect.top;
    const deltaBottom = rowRect.bottom - containerRect.bottom;
    if (deltaTop < 0) {
      container.scrollTop += deltaTop;
    } else if (deltaBottom > 0) {
      container.scrollTop += deltaBottom;
    }
  }, [selectedLayerId, layersForTree]);

  return (
    <div className="App">
      <header className="toolbar" data-tauri-drag-region>
        <div className="brand">
          <span className="brand-accent"></span>
          Layers Inspector
        </div>
        <div className="toolbar-actions" data-tauri-drag-region="false">
          <button
            className="icon-toggle"
            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
            title="Toggle theme"
            aria-label="Toggle theme"
            data-tauri-drag-region="false"
          >
            {theme === 'dark' ? '☀️' : '🌙'}
          </button>
          <div className={`status-pill ${connectionStatus}`} data-tauri-drag-region="false">
            <span className="status-dot" />
            {statusLabel}
          </div>
        </div>
      </header>

      <div
        className="pane-layout"
        style={{
          gridTemplateColumns: `${treeWidth}px 8px minmax(320px, 1fr) 8px ${rightWidth}px`,
        }}
      >
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
            <div className="panel-body scroll-body" ref={treeBodyRef}>
              {root && (() => {
                if (!rootEntry) {
                  return <div className="empty-state">No layers match your search.</div>;
                }
                return (
                  <Layer
                    key={root_id}
                    layer={rootEntry}
                    layers={layersForTree}
                    sendMessage={sendMessage}
                    setSelectedLayerId={setSelectedLayerId}
                    selectedLayerId={selectedLayerId}
                    isExpanded={isExpanded}
                    toggleExpanded={toggleExpanded}
                  />
                );
              })()}
            </div>
        </section>

        <div
          className="resize-handle"
          onMouseDown={(ev) => {
            resizing.current = { startX: ev.clientX, startWidth: treeWidth, target: 'tree' };
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
              <LayerDetails
                layer={selectedLayer}
                rootLayer={root}
                layers={layers}
                rootId={root_id}
                visibleSections={['identity', 'preview']}
              />
            ) : (
              <div className="empty-state">Select a layer from the tree.</div>
            )}
          </div>
        </section>

        <div
          className="resize-handle"
          onMouseDown={(ev) => {
            resizing.current = { startX: ev.clientX, startWidth: rightWidth, target: 'right' };
          }}
        />

        <section className="panel details-panel details-side-panel">
          <div className="panel-header">
            <div>
              <div className="panel-title">Layout & Appearance</div>
            </div>
            {selectedLayer && <div className="pill muted">#{selectedLayer[0]}</div>}
          </div>
          <div className="panel-body scroll-body">
            {selectedLayer ? (
              <LayerDetails
                layer={selectedLayer}
                rootLayer={root}
                layers={layers}
                rootId={root_id}
                visibleSections={['layout', 'appearance']}
              />
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
