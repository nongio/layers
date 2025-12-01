import React, { useState } from 'react';

function Layer({ layer, layers, sendMessage, setSelectedLayer, selectedLayer }) {
  let [expanded, setExpanded] = useState(true);
  let [highlighted, setHighlighted] = useState(false);

  if (!layer) return null;

  let id = layer[0];
  let attrs = layer[1];
  let children = layer[2];
  const label = attrs.key && attrs.key.trim() !== '' ? attrs.key : `[${id}]`;
  let selected = selectedLayer && selectedLayer[0] === id;

  const toggleHighlight = (ev) => {
    ev.stopPropagation();
    if (highlighted) {
      setHighlighted(false);
      sendMessage(["unhighlight", layer[3]]);
    } else {
      setHighlighted(true);
      sendMessage(["highlight", layer[3]]);
    }
  };

  return (
    <div
      className={`layer-row ${selected ? 'selected' : ''}`}
      onClick={(ev) => {
        ev.stopPropagation();
        setSelectedLayer(layer);
      }}
    >
      <div className="layer-meta">
        {children && children.length > 0 ? (
          <span
            className="caret"
            onClick={(ev) => {
              ev.stopPropagation();
              setExpanded(!expanded);
            }}
            aria-label={expanded ? "Collapse children" : "Expand children"}
          >
            {expanded ? "▾" : "▸"}
          </span>
        ) : (
          <span className="caret placeholder" />
        )}
        <span className="layer-name">{label}</span>
        <span
          className={`icon-toggle ${highlighted ? 'active' : ''}`}
          onClick={toggleHighlight}
          title={highlighted ? "Remove highlight in viewport" : "Highlight in viewport"}
        >
          {highlighted ? "●" : "○"}
        </span>
      </div>
      {expanded && children && children.length > 0 && (
        <div className="children">
          {children.map((childId) => {
            const childLayer = layers[childId];
            if (!childLayer) return null;
            return (
              <Layer
                key={childId}
                layer={childLayer}
                layers={layers}
                sendMessage={sendMessage}
                setSelectedLayer={setSelectedLayer}
                selectedLayer={selectedLayer}
              />
            );
          })}
        </div>
      )}
    </div>
  );
}

export default Layer;
