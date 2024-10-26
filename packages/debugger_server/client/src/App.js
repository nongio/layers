import React, { useEffect, useState } from 'react';
import Layer from './Layer';
import LayerDetails from './LayerDetails';

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

function connectWebSocket(socketInfo, setMessage, setSocket) {
  let url = `ws://${window.location.hostname}:${socketInfo.port}/ws/${socketInfo.uuid}`;
  const socket = new WebSocket(url);

  socket.onopen = function () {
    console.log('WebSocket connection established');
  };

  socket.onmessage = function (event) {
    // console.log('Message from server:', event.data);
    setMessage(event.data); // Update the state with the received message
  };

  socket.onclose = function () {
    console.log('WebSocket connection closed');
  };

  socket.onerror = function (error) {
    console.error('WebSocket error:', error);
  };
  console.log("set socket", socket)
  setSocket(socket); // Save the WebSocket instance to state
}

async function init(setMessage, setSocket) {
  console.log("Debugger client init");

  try {
    const websocket = await registerUser();
    connectWebSocket(websocket, setMessage, setSocket);
  } catch (error) {
    console.error('Error during initialization:', error);
  }
}


function App() {
  const [message, setMessage] = useState('');
  const [socket, setSocket] = useState(null);
  const [selectedLayer, setSelectedLayer] = useState(null);
  useEffect(() => {
    init(setMessage, setSocket);
    return () => {
      if (socket) {
        socket.close();
      }
    };
  }, []);
  var layers = null;
  var root = null;
  if (message !== '') {
    layers = JSON.parse(message);
    var root_id = layers[0];
    layers = layers[1];
    var root = layers[root_id];
    // console.log(root_id, root);
    // console.log("layers", layers);
  }

  const sendMessage = (msg) => {
    let stringy_msg = JSON.stringify(msg);
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(stringy_msg);
      console.log('Message sent:', stringy_msg);
    } else {
      console.error('WebSocket is not open');
    }
  };

  return (
    <div className="App">
      <div className='cols'>
        <div className='col'>
          <h1>
            Layers tree
          </h1>
          {root && <Layer key={root_id} layer={root} layers={layers} sendMessage={sendMessage} setSelectedLayer={setSelectedLayer} selectedLayer={selectedLayer} />}
        </div>
        <div className='col'></div>
        <div className='col sticky'>
          <h1>
            Details
            <div className="details">
              {selectedLayer && <LayerDetails layer={selectedLayer} />}
            </div>
          </h1>
        </div>
      </div >
    </div >
  );
}

export default App;