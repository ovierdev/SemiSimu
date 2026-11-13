use semifit_core::data::{IvCurve, csv_headers};
use semifit_core::extract::diode_log_fit;
use semifit_core::models::diode::current;
use semifit_core::spice::diode_model_card;
use wasm_bindgen::prelude::*;

/// Resultado visible desde JavaScript
#[wasm_bindgen(getter_with_clone)]
pub struct DiodeResult {
    pub is: f64,
    pub n: f64,
    pub r2: f64,
    pub points_used: usize,
    pub skipped: usize,
    pub model_card: String,
    pub v: Vec<f64>,
    pub i_meas: Vec<f64>,
    pub i_model: Vec<f64>,
}

/// Nombres de columnas para llenar los selectores
#[wasm_bindgen]
pub fn csv_columns(csv: &str) -> Vec<String> {
    csv_headers(csv)
}

#[wasm_bindgen]
pub fn extract_diode(
    csv: &str,
    v_col: usize,
    i_col: usize,
    t: f64,
    v_min: f64,
    v_max: f64,
) -> Result<DiodeResult, JsError> {
    let carga = IvCurve::from_csv_columns(csv, v_col, i_col)?;
    let curva = carga.curve;
    let r = diode_log_fit(&curva, t, v_min, v_max)?;
    let i_model = curva.v.iter().map(|&v| current(&r.params, v, t)).collect();

    Ok(DiodeResult {
        is: r.params.is,
        n: r.params.n,
        r2: r.r2,
        points_used: r.points_used,
        skipped: carga.skipped,
        model_card: diode_model_card("DFIT", &r.params),
        v: curva.v,
        i_meas: curva.i,
        i_model,
    })
}
