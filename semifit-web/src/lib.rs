use semifit_core::data::IvCurve;
use semifit_core::extract::diode_log_fit;
use semifit_core::spice::diode_model_card;
use wasm_bindgen::prelude::*;

/// Resultado visible desde JavaScript
#[wasm_bindgen(getter_with_clone)]
pub struct DiodeResult {
    pub is: f64,
    pub n: f64,
    pub r2: f64,
    pub points_used: usize,
    pub model_card: String,
}

/// Recibe el texto del CSV y devuelve los parámetros extraídos.
/// Si algo falla, JavaScript recibe una excepción con el mensaje de error.
#[wasm_bindgen]
pub fn extract_diode(csv: &str, t: f64, v_min: f64, v_max: f64) -> Result<DiodeResult, JsError> {
    let curva = IvCurve::from_csv_str(csv)?;
    let r = diode_log_fit(&curva, t, v_min, v_max)?;
    Ok(DiodeResult {
        is: r.params.is,
        n: r.params.n,
        r2: r.r2,
        points_used: r.points_used,
        model_card: diode_model_card("DFIT", &r.params),
    })
}
