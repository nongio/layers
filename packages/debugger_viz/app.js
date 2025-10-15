const listEl = document.getElementById("node-list");
const svgEl = document.getElementById("scene-canvas");
const detailsEl = document.getElementById("node-details");
const tooltipEl = document.getElementById("tooltip");
const sceneSizeEl = document.getElementById("scene-size");
const zoomSlider = document.getElementById("zoom-slider");
const toggleHidden = document.getElementById("toggle-hidden");
const toggleZero = document.getElementById("toggle-zero-size");
const reloadBtn = document.getElementById("reload-btn");
const fileLoader = document.getElementById("file-loader");
const template = document.getElementById("node-row");
const propertiesTable = document.querySelector(".properties-table");
const propertiesBody = document.getElementById("properties-body");
const propertiesEmpty = document.getElementById("properties-empty");

propertiesTable.style.display = "none";
propertiesEmpty.style.display = "block";

const DEFAULT_SCENE_PATH = "../../scene.json";

const ICON_EYE = `
<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
  <path d="M2.25 12s3.75-6 9.75-6 9.75 6 9.75 6-3.75 6-9.75 6-9.75-6-9.75-6Z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></path>
  <circle cx="12" cy="12" r="3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></circle>
</svg>`;

const ICON_EYE_OFF = `
<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
  <path d="m3.2 4.2 17 17" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"></path>
  <path d="M5.4 7.24C3.78 8.83 2.25 12 2.25 12s3.75 6 9.75 6c1.37 0 2.62-.24 3.75-.62" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M14.88 9.12A3.5 3.5 0 0 0 9.1 9.1" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M18.6 16.76C20.22 15.17 21.75 12 21.75 12s-3.75-6-9.75-6c-.62 0-1.21.05-1.78.13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></path>
</svg>`;

const state = {
  scene: null,
  collapsed: new Set(),
  selectedPath: null,
  nodes: new Map(),
  rows: new Map(),
  rects: new Map(),
  showHidden: toggleHidden.checked,
  showZero: toggleZero.checked,
  hiddenNodes: new Set(),
};

svgEl.style.transform = `scale(${zoomSlider.value})`;

async function loadSceneFromPath(path) {
  try {
    const response = await fetch(path, { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`Fetch failed: ${response.status} ${response.statusText}`);
    }
    const data = await response.json();
    setScene(data, path);
  } catch (err) {
    console.error("Failed to load scene", err);
    detailsEl.textContent = `Failed to load scene from ${path}. Use the file picker to load a JSON snapshot.`;
    updateProperties(null);
  }
}

function setScene(scene, source = "") {
  state.scene = scene;
  state.selectedPath = null;
  state.hiddenNodes.clear();
  sceneSizeEl.textContent = `Scene ${scene.size.width.toFixed(1)} × ${scene.size.height.toFixed(1)} ${source ? `(${source})` : ""}`;
  renderScene();
}

function renderScene() {
  if (!state.scene) {
    listEl.innerHTML = "";
    svgEl.innerHTML = "";
    detailsEl.textContent = "Load a scene.json to begin.";
    updateProperties(null);
    return;
  }

  state.nodes.clear();
  state.rows.clear();
  state.rects.clear();

  const fragment = document.createDocumentFragment();
  const svgFragment = document.createDocumentFragment();

  svgEl.setAttribute("viewBox", `0 0 ${state.scene.size.width} ${state.scene.size.height}`);
  svgEl.innerHTML = "";

  (state.scene.nodes ?? []).forEach((node, index) => {
    buildNode(node, [], 0, fragment, svgFragment, index, false);
  });

  listEl.innerHTML = "";
  listEl.appendChild(fragment);
  svgEl.appendChild(svgFragment);

  reselectNode();
}

function buildNode(node, parentPath, depth, listFragment, svgFragment, ordinal = 0, parentHidden = false) {
  const selfId =
    node.id !== undefined && node.id !== null
      ? String(node.id)
      : `${node.key ?? "node"}-${ordinal}`;
  const path = [...parentPath, selfId];
  const pathKey = path.join("/");

  state.nodes.set(pathKey, node);

  const bounds = node.global_bounds ?? node.render_layer?.bounds;
  const zeroSized = !bounds || bounds.width <= 0 || bounds.height <= 0;

  const explicitHidden = state.hiddenNodes.has(pathKey);
  const hidden = parentHidden || explicitHidden;

  if ((!state.showHidden && node.hidden) || (!state.showZero && zeroSized)) {
    return;
  }

  const rowFragment = template.content.cloneNode(true);
  const row = rowFragment.querySelector(".node-row");
  const toggle = row.querySelector(".node-toggle");
  const label = row.querySelector(".node-label");

  row.dataset.path = pathKey;
  row.dataset.depth = depth;
  row.style.paddingLeft = `${0.55 + depth * 1.1}rem`;

  const titleParts = [node.key || `node-${node.id ?? "?"}`];
  if (node.render_layer?.key && node.render_layer.key !== node.key) {
    titleParts.push(`(${node.render_layer.key})`);
  }
  label.textContent = titleParts.join(" ");

  if (node.hidden) {
    row.classList.add("hidden-node");
  }
  if (zeroSized) {
    row.classList.add("zero-size");
  }
  if (explicitHidden) {
    row.classList.add("hidden-by-user");
  }
  if (parentHidden) {
    row.classList.add("inherited-hidden");
  }

  const hasChildren = Boolean(node.children && node.children.length);
  if (hasChildren) {
    row.classList.add("has-children");
    toggle.textContent = state.collapsed.has(pathKey) ? "▶" : "▼";
    toggle.addEventListener("click", (event) => {
      event.stopPropagation();
      if (state.collapsed.has(pathKey)) {
        state.collapsed.delete(pathKey);
      } else {
        state.collapsed.add(pathKey);
      }
      renderScene();
    });
  } else {
    toggle.textContent = "•";
    toggle.classList.add("leaf");
  }

  const eyeBtn = row.querySelector(".node-eye");
  const iconHtml = explicitHidden ? ICON_EYE_OFF : ICON_EYE;
  eyeBtn.innerHTML = iconHtml;
  eyeBtn.classList.toggle("hidden", explicitHidden);
  eyeBtn.title = explicitHidden ? "Show layer" : "Hide layer";
  eyeBtn.setAttribute("aria-pressed", explicitHidden ? "true" : "false");
  eyeBtn.addEventListener("click", (event) => {
    event.stopPropagation();
    if (state.hiddenNodes.has(pathKey)) {
      state.hiddenNodes.delete(pathKey);
    } else {
      state.hiddenNodes.add(pathKey);
    }
    renderScene();
  });

  row.addEventListener("click", () => selectNode(pathKey, true));
  row.addEventListener("mouseenter", () => highlightRect(pathKey, true));
  row.addEventListener("mouseleave", () => highlightRect(pathKey, false));

  listFragment.appendChild(rowFragment);
  state.rows.set(pathKey, row);

  if (!zeroSized && bounds) {
    const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    rect.setAttribute("x", bounds.x ?? 0);
    rect.setAttribute("y", bounds.y ?? 0);
    rect.setAttribute("width", bounds.width ?? 0);
    rect.setAttribute("height", bounds.height ?? 0);
    rect.dataset.path = pathKey;
    if (hidden) {
      rect.style.display = "none";
      rect.classList.add("user-hidden");
    }
    rect.addEventListener("click", (event) => {
      event.stopPropagation();
      selectNode(pathKey, false);
    });
    rect.addEventListener("mouseenter", (event) => {
      highlightList(pathKey, true);
      showTooltip(event, node, bounds);
    });
    rect.addEventListener("mouseleave", () => {
      highlightList(pathKey, false);
      hideTooltip();
    });
    svgFragment.appendChild(rect);
    state.rects.set(pathKey, rect);
  }

  if (!state.collapsed.has(pathKey)) {
    (node.children ?? []).forEach((child, childIndex) => {
      buildNode(child, path, depth + 1, listFragment, svgFragment, childIndex, hidden);
    });
  }
}

function selectNode(pathKey, scrollIntoView) {
  if (state.selectedPath === pathKey) {
    return;
  }

  if (state.selectedPath) {
    state.rows.get(state.selectedPath)?.classList.remove("selected");
    state.rects.get(state.selectedPath)?.classList.remove("selected");
  }

  state.selectedPath = pathKey;

  state.rows.get(pathKey)?.classList.add("selected");
  const rect = state.rects.get(pathKey);
  if (rect) {
    rect.classList.add("selected");
  }

  if (scrollIntoView) {
    state.rows.get(pathKey)?.scrollIntoView({ block: "center", behavior: "smooth" });
  }

  updateDetails(pathKey);
  updateProperties(pathKey);
}

function reselectNode() {
  if (!state.selectedPath || !state.rows.has(state.selectedPath)) {
    state.selectedPath = null;
    detailsEl.textContent = "Select a node to inspect its properties.";
    updateProperties(null);
    return;
  }
  const path = state.selectedPath;
  state.selectedPath = null;
  selectNode(path, false);
}

function highlightRect(pathKey, hovered) {
  const rect = state.rects.get(pathKey);
  if (rect) {
    rect.classList.toggle("hovered", hovered);
  }
}

function highlightList(pathKey, hovered) {
  const row = state.rows.get(pathKey);
  if (row && !row.classList.contains("selected")) {
    row.classList.toggle("hovered", hovered);
  }
  highlightRect(pathKey, hovered);
  if (hovered) {
    row?.scrollIntoView({ block: "nearest" });
  }
}

function showTooltip(event, node, bounds) {
  tooltipEl.classList.remove("hidden");
  tooltipEl.textContent = `${node.key ?? node.render_layer?.key ?? "node"}\n${bounds.width.toFixed(1)} × ${bounds.height.toFixed(1)} at (${bounds.x.toFixed(1)}, ${bounds.y.toFixed(1)})`;
  const containerRect = svgEl.getBoundingClientRect();
  const x = event.clientX - containerRect.left;
  const y = event.clientY - containerRect.top;
  tooltipEl.style.left = `${x}px`;
  tooltipEl.style.top = `${y}px`;
}

function hideTooltip() {
  tooltipEl.classList.add("hidden");
}

function updateDetails(pathKey) {
  const node = state.nodes.get(pathKey);
  if (!node) {
    detailsEl.textContent = "Select a node to inspect.";
    return;
  }
  const bounds = node.global_bounds ?? node.render_layer?.bounds;
  const lines = [
    `key: ${node.key ?? "(none)"}`,
    `id: ${node.id ?? "-"}`,
    `opacity: ${node.opacity ?? node.render_layer?.opacity ?? "-"}`,
    `hidden: ${node.hidden ? "yes" : "no"}`,
    `pointer_events: ${node.pointer_events}`,
  ];
  if (bounds) {
    lines.push(
      `global_bounds: x=${bounds.x?.toFixed(2)}, y=${bounds.y?.toFixed(2)}, w=${bounds.width?.toFixed(2)}, h=${bounds.height?.toFixed(2)}`,
    );
  } else {
    lines.push("global_bounds: (not available)");
  }
  if (node.render_layer?.blend_mode) {
    lines.push(`blend_mode: ${node.render_layer.blend_mode}`);
  }
  detailsEl.textContent = lines.join("\n");
}

function updateProperties(pathKey) {
  propertiesBody.innerHTML = "";
  if (!pathKey) {
    propertiesEmpty.style.display = "block";
    propertiesTable.style.display = "none";
    return;
  }

  const node = state.nodes.get(pathKey);
  if (!node) {
    propertiesEmpty.style.display = "block";
    propertiesTable.style.display = "none";
    return;
  }

  propertiesEmpty.style.display = "none";
  propertiesTable.style.display = "table";

  const addRow = (label, value) => {
    const tr = document.createElement("tr");
    const th = document.createElement("th");
    th.textContent = label;
    const td = document.createElement("td");
    td.textContent = value;
    tr.append(th, td);
    propertiesBody.appendChild(tr);
  };

  const renderLayer = node.render_layer ?? {};

  addRow("Key", node.key ?? "—");
  addRow("ID", node.id ?? "—");
  addRow("Children", (node.children?.length ?? 0).toString());
  addRow("Hidden", formatBool(node.hidden));
  addRow("Pointer events", formatBool(node.pointer_events));
  addRow("Opacity", formatNumber(node.opacity ?? renderLayer.opacity));
  addRow("Needs layout", formatBool(node.needs_layout));
  addRow("Needs repaint", formatBool(node.needs_repaint));
  addRow("Image cached", formatBool(node.image_cached));
  addRow("Picture cached", formatBool(node.picture_cached));
  addRow("Local bounds", formatRect(node.local_bounds));
  addRow("Global bounds", formatRect(node.global_bounds));
  addRow("Render layer key", renderLayer.key ?? "—");
  addRow("Render layer bounds", formatRect(renderLayer.bounds));
  addRow("Transformed bounds", formatRect(renderLayer.transformed_bounds));
  addRow("Blend mode", renderLayer.blend_mode ?? "—");
  addRow("Background", describeColor(renderLayer.background_color));
  addRow("Border width", formatNumber(renderLayer.border_width));
  addRow("Shadow radius", formatNumber(renderLayer.shadow_radius));
  addRow("Shadow offset", formatPoint(renderLayer.shadow_offset));
}

function formatBool(value) {
  if (value === undefined || value === null) {
    return "—";
  }
  return value ? "yes" : "no";
}

function formatNumber(value) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "—";
  }
  return Number(value).toFixed(3);
}

function formatRect(rect) {
  if (!rect) {
    return "—";
  }
  const x = formatShortNumber(rect.x);
  const y = formatShortNumber(rect.y);
  const width = formatShortNumber(rect.width);
  const height = formatShortNumber(rect.height);
  return `x=${x}, y=${y}, w=${width}, h=${height}`;
}

function formatPoint(point) {
  if (!point) {
    return "—";
  }
  const x = formatShortNumber(point.x);
  const y = formatShortNumber(point.y);
  return `x=${x}, y=${y}`;
}

function formatShortNumber(value) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "—";
  }
  return Number(value).toFixed(2);
}

function describeColor(color) {
  if (!color) {
    return "—";
  }
  if (typeof color === "string") {
    return color;
  }
  if (color.Solid?.color) {
    const c = color.Solid.color;
    const l = formatShortNumber(c.l);
    const a = formatShortNumber(c.a);
    const b = formatShortNumber(c.b);
    const alpha = formatShortNumber(c.alpha);
    return `l=${l}, a=${a}, b=${b}, alpha=${alpha}`;
  }
  return JSON.stringify(color);
}

zoomSlider.addEventListener("input", (event) => {
  const scale = Number(event.target.value);
  svgEl.style.transform = `scale(${scale})`;
});

toggleHidden.addEventListener("change", (event) => {
  state.showHidden = event.target.checked;
  renderScene();
});

toggleZero.addEventListener("change", (event) => {
  state.showZero = event.target.checked;
  renderScene();
});

reloadBtn.addEventListener("click", () => {
  loadSceneFromPath(DEFAULT_SCENE_PATH);
});

fileLoader.addEventListener("change", (event) => {
  const file = event.target.files?.[0];
  if (!file) {
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    try {
      const data = JSON.parse(reader.result);
      state.collapsed.clear();
      setScene(data, file.name);
    } catch (err) {
      detailsEl.textContent = `Failed to parse ${file.name}: ${err.message}`;
      updateProperties(null);
    }
  };
  reader.readAsText(file);
});

// Attempt auto-load on startup
loadSceneFromPath(DEFAULT_SCENE_PATH);
