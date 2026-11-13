import init, { extract_diode } from "./pkg/semifit_web.js";
    await init();

    const $ = (id) => document.getElementById(id);
    let csvTexto = null;

    $("archivo").addEventListener("change", async (e) => {
      csvTexto = await e.target.files[0].text();
      actualizar();
    });
    for (const id of ["temp", "vmin", "vmax"]) $(id).addEventListener("input", actualizar);
    $("escalaLog").addEventListener("change", actualizar);
    $("copiar").addEventListener("click", () =>
      navigator.clipboard.writeText($("tarjeta").textContent));

    function actualizar() {
      if (!csvTexto) return;
      const t = parseFloat($("temp").value);
      const vmin = parseFloat($("vmin").value);
      const vmax = parseFloat($("vmax").value);

      let r;
      try {
        r = extract_diode(csvTexto, t, vmin, vmax);
      } catch (err) {
        $("estado").textContent = "Error: " + err.message;
        return;
      }
      $("estado").textContent = "";

      $("params").textContent =
        `Is = ${r.is.toExponential(4)} A\n` +
        `n  = ${r.n.toFixed(4)}\n` +
        `R² = ${r.r2.toFixed(6)}\n` +
        `Puntos en ventana: ${r.points_used}`;
      $("tarjeta").textContent = r.model_card;

      const v = Array.from(r.v);
      const iMeas = Array.from(r.i_meas);
      const iModel = Array.from(r.i_model);
      r.free(); // libera la memoria del lado de Rust

      Plotly.react("grafica", [
        { x: v, y: iMeas, mode: "markers", name: "Medición", marker: { size: 6 } },
        { x: v, y: iModel, mode: "lines", name: "Modelo", line: { width: 2 } },
      ], {
        xaxis: { title: { text: "V [V]" } },
        yaxis: {
          title: { text: "I [A]" },
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
