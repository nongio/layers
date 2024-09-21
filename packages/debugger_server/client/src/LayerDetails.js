import React, { useEffect, useState } from 'react';
import { oklab, formatRgb } from 'culori';


function LayerDetails({ layer }) {
    let id = layer[0];
    let attrs = layer[1];


    let name = `[${id}] ${attrs.key}`;
    let background_color = formatRgb(oklab(attrs.background_color["Solid"]["color"]));
    let border_color = formatRgb(oklab(attrs.border_color["Solid"]["color"]));
    let border_width = attrs.border_width;
    let border_radius = attrs.border_corner_radius.top_left;
    let opacity = attrs.opacity;
    let shadow_offset = attrs.shadow_offset;
    let shadow_radius = attrs.shadow_radius;
    let shadow_color = formatRgb(oklab(attrs.shadow_color));
    return (
        <div className={`layer-details`}>
            <span className="name">{name}</span>
            <div className="attrs">
                <div className="attr-field">key: {attrs.key}</div>
                <div className="attr-field">style size: {JSON.stringify(attrs.size)}</div>
                <div className="attr-field">bounds: {JSON.stringify(attrs.bounds)}</div>
                <div className="attr-field">T bounds: {JSON.stringify(attrs.transformed_bounds)}</div>
                <div className="attr-field">bounds_with_children: {JSON.stringify(attrs.bounds_with_children)}</div>
                <div className="attr-field">background_color: {background_color}</div>
                <div className="attr-field">border_style: {attrs.border_style}</div>
                <div className="attr-field">border_color: {border_color}</div>
                <div className="attr-field">border_width: {border_width}</div>
                <div className="attr-field">border_radius: {border_radius}</div>
                <div className="attr-field">opacity: {opacity}</div>
                <div className="attr-field">shadow_offset: {JSON.stringify(shadow_offset)}</div>
                <div className="attr-field">shadow_radius: {shadow_radius}</div>
                <div className="attr-field">shadow_color: {shadow_color}</div>
                <div className="attr-field">blend mode: {attrs.blend_mode}</div>
            </div>

            <div className="preview">
                <div className="layer-preview" style={{
                    backgroundColor: background_color,
                    borderColor: border_color,
                    borderWidth: border_width,
                    borderRadius: border_radius,
                    opacity: opacity,
                    boxShadow: `${shadow_offset.x}px ${shadow_offset.y}px ${shadow_radius}px ${shadow_color}`
                }}>{attrs.key}</div>
            </div>
        </div>
    );
}

export default LayerDetails;