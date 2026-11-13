import init, { csv_columns, extract_diode } from "./pkg/semifit_web.js";
await init(); // carga el .wasm

const $ = (id) => document.getElementById(id);
let csvTexto = null;
let nombres = [];

// Propone una columna buscando patrones en los nombres; el usuario puede cambiarla
function adivinar(patrones, porDefecto) {
  for (const p of patrones) {
    const k = nombres.findIndex((n) => p.test(n));
    if (k >= 0) return k;
  }
  return porDefecto;
}

function llenarSelector(id, seleccion) {
  $(id).innerHTML = nombres
    .map((n, k) => `<option value="${k}" ${k === seleccion ? "selected" : ""}>${n}</option>`)
    .join("");
}

// --- Eventos ---
$("archivo").addEventListener("change", async (e) => {
  csvTexto = await e.target.files[0].text();
  nombres = csv_columns(csvTexto);
  llenarSelector("colV", adivinar([/v.*meas/i, /volt/i, /^v/i], 0));
  llenarSelector("colI", adivinar([/i.*meas/i, /curr/i, /^i/i], Math.min(1, nombres.length - 1)));
  actualizar();
});
$("colV").addEventListener("change", actualizar);
$("colI").addEventListener("change", actualizar);
for (const id of ["temp", "vmin", "vmax"]) $(id).addEventListener("input", actualizar);
$("escalaLog").addEventListener("change", actualizar);
$("copiar").addEventListener("click", () =>
  navigator.clipboard.writeText($("tarjeta").textContent));

// --- Cálculo y gráfica ---
function actualizar() {
  if (!csvTexto) return;

  const vCol = parseInt($("colV").value);
  const iCol = parseInt($("colI").value);
  if (vCol === iCol) {
    $("estado").textContent = "Elige columnas distintas para V e I.";
    return;
  }

  const t = parseFloat($("temp").value);
  const vmin = parseFloat($("vmin").value);
  const vmax = parseFloat($("vmax").value);

  let r;
  try {
    r = extract_diode(csvTexto, vCol, iCol, t, vmin, vmax);
  } catch (err) {
    $("estado").textContent = "Error: " + err.message;
    return;
  }
  $("estado").textContent = "";

  $("params").textContent =
    `Is = ${r.is.toExponential(4)} A\n` +
    `n  = ${r.n.toFixed(4)}\n` +
    `R² = ${r.r2.toFixed(6)}\n` +
    `Puntos en ventana: ${r.points_used}\n` +
    `Filas descartadas (NaN/inf): ${r.skipped}`;
  $("tarjeta").textContent = r.model_card;

  const v = Array.from(r.v);
  const iMeas = Array.from(r.i_meas);
  const iModel = Array.from(r.i_model);
  r.free(); // libera la memoria del lado de Rust

  Plotly.react("grafica", [
    { x: v, y: iMeas, mode: "markers", name: "Medición", marker: { size: 6 } },
    { x: v, y: iModel, mode: "lines", name: "Modelo", line: { width: 2 } },
  ], {
    xaxis: { title: { text: `${nombres[vCol]} [V]` } },
    yaxis: {
      title: { text: `${nombres[iCol]} [A]` },
      type: $("escalaLog").checked ? "log" : "linear",
      exponentformat: "e",
    },
    shapes: [{
      type: "rect", xref: "x", yref: "paper",
      x0: vmin, x1: vmax, y0: 0, y1: 1,
      fillcolor: "rgba(80, 130, 255, 0.12)", line: { width: 0 },
    }],
    margin: { t: 20 },
    legend: { x: 0.02, y: 0.98 },
  });
}
