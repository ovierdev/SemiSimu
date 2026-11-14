use semifit_core::data::{IvCurve, csv_headers};
use semifit_core::extract::diode_log_fit;
use semifit_core::models::diode::current;
use semifit_core::spice::diode_model_card;
use wasm_bindgen::prelude::*;

/// Datos medidos, listos para graficar
#[wasm_bindgen(getter_with_clone)]
pub struct CurveData {
    pub v: Vec<f64>,
    pub i: Vec<f64>,
    pub skipped: usize,
}

/// Resultado del ajuste
#[wasm_bindgen(getter_with_clone)]
pub struct DiodeResult {
    pub is: f64,
    pub n: f64,
    pub r2: f64,
    pub rms_pct: f64,
    pub points_used: usize,
    pub model_card: String,
    /// Corriente del modelo en los mismos voltajes que CurveData.v
    pub i_model: Vec<f64>,
    pub warnings: Vec<String>,
}

#[wasm_bindgen]
pub fn csv_columns(csv: &str) -> Vec<String> {
    csv_headers(csv)
}

#[wasm_bindgen]
pub fn load_curve(csv: &str, v_col: usize, i_col: usize) -> Result<CurveData, JsError> {
    let carga = IvCurve::from_csv_columns(csv, v_col, i_col)?;
    Ok(CurveData {
        v: carga.curve.v,
        i: carga.curve.i,
        skipped: carga.skipped,
    })
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
    let curva = IvCurve::from_csv_columns(csv, v_col, i_col)?.curve;
    let r = diode_log_fit(&curva, t, v_min, v_max)?;
    let i_model = curva.v.iter().map(|&v| current(&r.params, v, t)).collect();

    Ok(DiodeResult {
        is: r.params.is,
        n: r.params.n,
        r2: r.r2,
        rms_pct: r.rms_pct,
        points_used: r.points_used,
        model_card: diode_model_card("DFIT", &r.params),
        i_model,
        warnings: r.warnings.iter().map(|w| w.to_string()).collect(),
    })
}
