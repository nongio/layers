import React, { useEffect, useState } from 'react';


function Layer({ layer, layers, sendMessage, setSelectedLayer, selectedLayer }) {
    let id = layer[0];
    let attrs = layer[1];
    let children = layer[2];
    let [expanded, setExpanded] = useState(true);
    let [highlighted, setHighlighted] = useState(false);
    let name = `[${id}] ${attrs.key}`;
    let selected = selectedLayer && selectedLayer[0] === id;
    return (
        <div className={`layer ${(selected ? 'highlight' : '')} layer-${id}`} onClick={(ev) => {
            ev.stopPropagation();
            setSelectedLayer(layer);

        }}>
            {children && children.length > 0 &&
                <span className="expand" onClick={() => setExpanded(!expanded)}>{expanded ? "⬇️" : "➡️"}</span>
            }
            <span className="name">{name}</span>
            <span className="highlight" onClick={() => {
                if (highlighted) {
                    setHighlighted(false);
                    sendMessage(["unhighlight", layer[3]])
                } else {
                    setHighlighted(true);
                    sendMessage(["highlight", layer[3]])
                }
            }}>{highlighted ? "👁️" : "👄"}</span>
            {false && <div className="attrs">{JSON.stringify(attrs)}</div>}
            {expanded && children && children.length > 0 && <div className="children">
                {children.map((id) => {

                    return <Layer key={id} layer={layers[id]} layers={layers} sendMessage={sendMessage} setSelectedLayer={setSelectedLayer} selectedLayer={selectedLayer} />
                })}
            </div>}
        </div>
    );
}

export default Layer;