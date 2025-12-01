import React from 'react';
import { oklab, formatRgb } from 'culori';

function asColor(value) {
  if (!value || !value.Solid) return null;
  return formatRgb(oklab(value.Solid.color));
}

function safeFormatRect(rect) {
  if (!rect) return '-';
  return `${Math.round(rect.x)}, ${Math.round(rect.y)} • ${Math.round(rect.width)} × ${Math.round(rect.height)}`;
}

function LayerDetails({ layer, rootLayer, layers, rootId }) {
  let id = layer[0];
  let attrs = layer[1];
  let visible = attrs.visible ?? true;
  const [openSections, setOpenSections] = React.useState({
    identity: true,
    layout: false,
    appearance: false,
    preview: true,
  });
  const [zoom, setZoom] = React.useState(1);
  const parentChain = React.useMemo(() => {
    if (!layers) return [];
    const parentMap = {};
    Object.values(layers).forEach((entry) => {
      const children = entry?.[2] ?? [];
      children.forEach((cid) => {
        parentMap[cid] = entry[0];
      });
    });
    const chain = [];
    let current = id;
    while (current && parentMap[current] !== undefined) {
      const parentId = parentMap[current];
      chain.push(parentId);
      if (parentId === rootId) break;
      current = parentId;
    }
    return chain;
  }, [layers, id, rootId]);

  const toggleSection = (key) => {
    setOpenSections((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  let backgroundColor = asColor(attrs.background_color);
  let borderColor = asColor(attrs.border_color);
  let borderWidth = attrs.border_width ?? 0;
  let borderStyle = attrs.border_style ?? 'solid';
  let borderRadius = attrs.border_corner_radius?.top_left ?? 0;
  let opacity = attrs.opacity;
  let shadowOffset = attrs.shadow_offset ?? { x: 0, y: 0 };
  let shadowRadius = attrs.shadow_radius ?? 0;
  let shadowColor = attrs.shadow_color ? formatRgb(oklab(attrs.shadow_color)) : 'rgba(0,0,0,0.25)';
  let transformedBounds = attrs.transformed_bounds;

  let rootBounds = rootLayer?.[1]?.bounds || rootLayer?.[1]?.transformed_bounds;
  let viewWidth = rootBounds?.width ?? 400;
  let viewHeight = rootBounds?.height ?? 400;
  let rootX = rootBounds?.x ?? 0;
  let rootY = rootBounds?.y ?? 0;
  let minX = rootX;
  let minY = rootY;
  let maxX = rootX + viewWidth;
  let maxY = rootY + viewHeight;
  if (transformedBounds) {
    minX = Math.min(minX, transformedBounds.x);
    minY = Math.min(minY, transformedBounds.y);
    maxX = Math.max(maxX, transformedBounds.x + transformedBounds.width);
    maxY = Math.max(maxY, transformedBounds.y + transformedBounds.height);
  }
  const pad = Math.max(viewWidth, viewHeight) * 0.1;
  const paddedMinX = minX - pad;
  const paddedMinY = minY - pad;
  const paddedWidth = (maxX - minX) + pad * 2;
  const paddedHeight = (maxY - minY) + pad * 2;
  const centerX = paddedMinX + paddedWidth / 2;
  const centerY = paddedMinY + paddedHeight / 2;
  let viewBoxWidth = paddedWidth / zoom;
  let viewBoxHeight = paddedHeight / zoom;
  // Keep the root viewport always visible.
  viewBoxWidth = Math.max(viewBoxWidth, viewWidth);
  viewBoxHeight = Math.max(viewBoxHeight, viewHeight);
  let viewBoxX = centerX - viewBoxWidth / 2;
  let viewBoxY = centerY - viewBoxHeight / 2;
  // Clamp so root stays inside the viewBox.
  if (viewBoxX > rootX) {
    viewBoxX = rootX;
  }
  if (viewBoxX + viewBoxWidth < rootX + viewWidth) {
    viewBoxX = rootX + viewWidth - viewBoxWidth;
  }
  if (viewBoxY > rootY) {
    viewBoxY = rootY;
  }
  if (viewBoxY + viewBoxHeight < rootY + viewHeight) {
    viewBoxY = rootY + viewHeight - viewBoxHeight;
  }
  const filterId = `shadow-${id}`;
  const hasShadow = shadowRadius > 0 || shadowOffset.x !== 0 || shadowOffset.y !== 0;
  // no-op: layerFill removed

  return (
    <div className={`layer-details`}>
      <div className="section identity-section">
        <button className="section-toggle" onClick={() => toggleSection('identity')}>
          <span className="chevron">{openSections.identity ? '▾' : '▸'}</span>
          <span className="section-title">Identity</span>
        </button>
        {openSections.identity && (
          <>
            <div className="kv-row">
              <span className="label">Key</span>
              <span className="value">{attrs.key}</span>
            </div>
            <div className="kv-row">
              <span className="label">Layer ID</span>
              <span className="value">[{id}]</span>
            </div>
            <div className="kv-row">
              <span className="label">Visibility</span>
              <span className={`badge ${visible ? 'green' : 'red'}`}>{visible ? 'Visible' : 'Hidden'}</span>
            </div>
          </>
        )}
      </div>

      <div className="section">
        <button className="section-toggle" onClick={() => toggleSection('layout')}>
          <span className="chevron">{openSections.layout ? '▾' : '▸'}</span>
          <span className="section-title">Layout</span>
        </button>
        {openSections.layout && (
          <>
            <div className="kv-row">
              <span className="label">Style size</span>
              <span className="value code">{attrs.size ? `${attrs.size.x} × ${attrs.size.y}` : '-'}</span>
            </div>
            <div className="kv-row">
              <span className="label">Bounds</span>
              <span className="value code">{safeFormatRect(attrs.bounds)}</span>
            </div>
            <div className="kv-row">
              <span className="label">Transformed</span>
              <span className="value code">{safeFormatRect(attrs.transformed_bounds)}</span>
            </div>
            <div className="kv-row">
              <span className="label">With children</span>
              <span className="value code">{safeFormatRect(attrs.bounds_with_children)}</span>
            </div>
          </>
        )}
      </div>

      <div className="section">
        <button className="section-toggle" onClick={() => toggleSection('appearance')}>
          <span className="chevron">{openSections.appearance ? '▾' : '▸'}</span>
          <span className="section-title">Appearance</span>
        </button>
        {openSections.appearance && (
          <>
            <div className="kv-row preview-inline">
              <div className="layer-preview square"
                style={{
                  backgroundColor: backgroundColor ?? 'transparent',
                  borderColor: borderColor ?? 'transparent',
                  borderWidth: borderWidth,
                  borderRadius: borderRadius,
                  opacity: opacity,
                  boxShadow: `${shadowOffset.x}px ${shadowOffset.y}px ${shadowRadius}px ${shadowColor}`,
                  borderStyle: borderStyle
                }}
              >
                {attrs.key}
              </div>
            </div>
            <div className="kv-row">
              <span className="label">Background</span>
              <span className="value swatch-row">
                {backgroundColor && <span className="swatch" style={{ background: backgroundColor }} />}
                <span className="code">{backgroundColor ?? 'none'}</span>
              </span>
            </div>
            <div className="kv-row">
              <span className="label">Border</span>
              <span className="value swatch-row">
                {borderColor && <span className="swatch" style={{ background: borderColor }} />}
                <span className="code">
                  {borderStyle} {borderWidth}px
                </span>
              </span>
            </div>
            <div className="kv-row">
              <span className="label">Radius</span>
              <span className="value code">{borderRadius}px</span>
            </div>
            <div className="kv-row">
              <span className="label">Opacity</span>
              <span className="value code">{opacity}</span>
            </div>
            <div className="kv-row">
              <span className="label">Shadow</span>
              <span className="value code">
                {shadowOffset ? `${shadowOffset.x}, ${shadowOffset.y}` : '-'} / {shadowRadius}px
              </span>
            </div>
            <div className="kv-row">
              <span className="label">Shadow color</span>
              <span className="value swatch-row">
                <span className="swatch" style={{ background: shadowColor }} />
                <span className="code">{shadowColor}</span>
              </span>
            </div>
            <div className="kv-row">
              <span className="label">Blend mode</span>
              <span className="value code">{attrs.blend_mode}</span>
            </div>
          </>
        )}
      </div>

      <div className="section preview">
        <button className="section-toggle" onClick={() => toggleSection('preview')}>
          <span className="chevron">{openSections.preview ? '▾' : '▸'}</span>
          <span className="section-title">Preview</span>
        </button>
        {openSections.preview && (
          <>
            <div className="preview-toolbar">
              <div className="zoom-controls">
                <button className="ghost-btn" onClick={() => setZoom((z) => Math.min(z * 1.25, 8))}>+</button>
                <button className="ghost-btn" onClick={() => setZoom((z) => Math.max(z / 1.25, 0.25))}>-</button>
              </div>
            </div>
            <div className="svg-preview">
              <svg viewBox={`${viewBoxX} ${viewBoxY} ${viewBoxWidth} ${viewBoxHeight}`} preserveAspectRatio="xMinYMin meet">
                {hasShadow && (
                  <defs>
                    <filter id={filterId} x="-50%" y="-50%" width="200%" height="200%">
                      <feDropShadow
                        dx={shadowOffset.x}
                        dy={shadowOffset.y}
                        stdDeviation={Math.max(shadowRadius, 0.5) / 2}
                        floodColor={shadowColor}
                        floodOpacity="1"
                      />
                    </filter>
                  </defs>
                )}
                <rect className="svg-root" x={rootX} y={rootY} width={viewWidth} height={viewHeight} />
                <rect className="svg-viewport" x={rootX} y={rootY} width={viewWidth} height={viewHeight} />
                <text className="svg-label" x={rootX + 6} y={rootY + 14}>viewport</text>
                {transformedBounds && parentChain.map((pid) => {
                  const parentLayer = layers?.[pid];
                  const pb = parentLayer?.[1]?.transformed_bounds;
                  if (!pb) return null;
                  return (
                    <rect
                      key={`anc-${pid}`}
                      className="svg-ancestor"
                      x={pb.x}
                      y={pb.y}
                      width={pb.width}
                      height={pb.height}
                    />
                  );
                })}
                {transformedBounds && (
                  <rect
                    x={transformedBounds.x}
                    y={transformedBounds.y}
                    width={transformedBounds.width}
                    height={transformedBounds.height}
                    className="svg-layer"
                  />
                )}
              </svg>
              <div className="preview-caption">Transformed bounds in root space</div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default LayerDetails;
