import init, { csv_columns, load_curve, extract_diode } from "./pkg/semifit_web.js";
await init();

const $ = (id) => document.getElementById(id);
let csvTexto = null;
let nombres = [];

// --- Selectores de columnas ---

function adivinar(patrones, porDefecto) {
  for (const p of patrones) {
    const k = nombres.findIndex((n) => p.test(n));
    if (k >= 0) return k;
  }
  return porDefecto;
}

function llenarSelector(id, seleccion) {
  // new Option() trata el nombre como texto, nunca como HTML
  $(id).replaceChildren(...nombres.map((n, k) => new Option(n, k, false, k === seleccion)));
}

// Si el nombre de la columna trae la temperatura ("at 17C", "300 K"), la usa
function temperaturaDeColumna(nombre) {
  const m = nombre.match(/(-?\d+(?:\.\d+)?)\s*°?\s*([CK])\b/i);
  if (!m) return null;
  const valor = parseFloat(m[1]);
  return m[2].toUpperCase() === "C" ? valor + 273.15 : valor;
}

function aplicarTemperatura() {
  const t = temperaturaDeColumna(nombres[parseInt($("colI").value)] ?? "");
  if (t !== null) $("temp").value = t.toFixed(2);
}

// --- Eventos ---

$("archivo").addEventListener("change", async (e) => {
  csvTexto = await e.target.files[0].text();
  nombres = csv_columns(csvTexto);
  llenarSelector("colV", adivinar([/v.*meas/i, /volt/i, /^v/i], 0));
  llenarSelector("colI", adivinar([/i.*meas/i, /curr/i, /^i/i], Math.min(1, nombres.length - 1)));
  aplicarTemperatura();
  actualizar();
});
$("colV").addEventListener("change", actualizar);
$("colI").addEventListener("change", () => {
  aplicarTemperatura();
  actualizar();
});
for (const id of ["temp", "vmin", "vmax"]) $(id).addEventListener("input", actualizar);
$("escalaLog").addEventListener("change", actualizar);
$("copiar").addEventListener("click", () =>
  navigator.clipboard.writeText($("tarjeta").textContent));

// --- Cálculo y gráfica ---

function limpiarResultados() {
  $("params").textContent = "—";
  $("tarjeta").textContent = "—";
  $("avisos").replaceChildren();
}

function mostrarAvisos(lista) {
  $("avisos").replaceChildren(...lista.map((texto) => {
    const li = document.createElement("li");
    li.textContent = texto;
    return li;
  }));
}

function actualizar() {
  if (!csvTexto) return;

  const vCol = parseInt($("colV").value);
  const iCol = parseInt($("colI").value);
  if (vCol === iCol) {
    $("estado").textContent = "Elige columnas distintas para V e I.";
    limpiarResultados();
    Plotly.purge("grafica");
    return;
  }

  const t = parseFloat($("temp").value);
  const vmin = parseFloat($("vmin").value);
  const vmax = parseFloat($("vmax").value);

  // 1) Datos: se grafican aunque el ajuste falle
  let datos;
  try {
    datos = load_curve(csvTexto, vCol, iCol);
  } catch (err) {
    $("estado").textContent = "Error: " + err.message;
    limpiarResultados();
    Plotly.purge("grafica");
    return;
  }
  const v = Array.from(datos.v);
  const iMeas = Array.from(datos.i);
  const descartadas = datos.skipped;
  datos.free();

  const trazas = [
    { x: v, y: iMeas, mode: "markers", name: "Medición", marker: { size: 6 } },
  ];

  // 2) Ajuste: si falla, se avisa pero la gráfica de datos se mantiene
  $("estado").textContent = "";
  try {
    const r = extract_diode(csvTexto, vCol, iCol, t, vmin, vmax);
    trazas.push({ x: v, y: Array.from(r.i_model), mode: "lines", name: "Modelo", line: { width: 2 } });

    $("params").textContent =
      `Is = ${r.is.toExponential(4)} A\n` +
      `n  = ${r.n.toFixed(4)}\n` +
      `R² = ${r.r2.toFixed(6)}\n` +
      `Error RMS = ${r.rms_pct.toFixed(1)} %\n` +
      `Puntos en ventana: ${r.points_used}\n` +
      `Filas descartadas (NaN/inf): ${descartadas}`;
    $("tarjeta").textContent = r.model_card;
    mostrarAvisos(r.warnings);
    r.free();
  } catch (err) {
    $("estado").textContent = "Error: " + err.message;
    limpiarResultados();
  }

  Plotly.react("grafica", trazas, {
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
