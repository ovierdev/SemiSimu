use crate::constants::thermal_voltage;
use crate::data::IvCurve;
use crate::fit::linear_regression;
use crate::models::diode::DiodeParams;
use std::fmt;

#[derive(Debug)]
pub enum ExtractError {
    TooFewPoints { found: usize },
    NonPositiveCurrent { v: f64, i: f64 },
    Degenerate,
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::TooFewPoints { found } => {
                write!(
                    f,
                    "solo {found} puntos en la ventana; se necesitan al menos 3"
                )
            }
            ExtractError::NonPositiveCurrent { v, i } => {
                write!(
                    f,
                    "Corriente no positiva ({i:e} A en {v} V); no se pude tomer ln(I)"
                )
            }
            ExtractError::Degenerate => write!(f, "los datos no forman una exponencial creciente"),
        }
    }
}

impl std::error::Error for ExtractError {}

// Resultado de la extraccion, con metricas de calidad
#[derive(Debug, Clone, Copy)]
pub struct DiodeExtraction {
    pub params: DiodeParams,
    pub r2: f64,
    pub points_used: usize,
}

// Extrae Is y n ajustado ln(I) vs V dentro de la ventana [v_min, v_max].
// Valido en directa, donde V>> n*V_T y el "-1" de Shockley es despreciable.
pub fn diode_log_fit(
    curve: &IvCurve,
    t: f64,
    v_min: f64,
    v_max: f64,
) -> Result<DiodeExtraction, ExtractError> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();

    for (&v, &i) in curve.v.iter().zip(&curve.i) {
        if v < v_min || v > v_max {
            continue;
        }
        if i <= 0.0 {
            return Err(ExtractError::NonPositiveCurrent { v, i });
        }
        xs.push(v);
        ys.push(i.ln());
    }

    if xs.len() < 3 {
        return Err(ExtractError::TooFewPoints { found: xs.len() });
    }

    let fit = linear_regression(&xs, &ys).ok_or(ExtractError::Degenerate)?;
    if fit.slope <= 0.0 {
        return Err(ExtractError::Degenerate);
    }

    let vt = thermal_voltage(t);
    Ok(DiodeExtraction {
        params: DiodeParams {
            is: fit.intercept.exp(),
            n: 1.0 / (fit.slope * vt),
        },
        r2: fit.r2,
        points_used: xs.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::diode::current;

    fn curva_ideal(p: &DiodeParams, t: f64) -> IvCurve {
        let v: Vec<f64> = (0..=80).map(|k| k as f64 * 0.01).collect();
        let i = v.iter().map(|&x| current(p, x, t)).collect();
        IvCurve { v, i }
    }

    #[test]
    fn recupera_parametros_sin_ruido() {
        let real = DiodeParams {
            is: 2.5e-14,
            n: 1.08,
        };
        let c = curva_ideal(&real, 300.0);
        let r = diode_log_fit(&c, 300.0, 0.3, 0.7).unwrap();
        assert!((r.params.n / real.n - 1.0).abs() < 1e-3);
        assert!((r.params.is / real.is - 1.0).abs() < 1e-3);
    }

    #[test]
    fn rechaza_corriente_cero() {
        let real = DiodeParams {
            is: 2.5e-14,
            n: 1.08,
        };
        let c = curva_ideal(&real, 300.0); // incluye V = 0 → I = 0
        assert!(matches!(
            diode_log_fit(&c, 300.0, 0.0, 0.5),
            Err(ExtractError::NonPositiveCurrent { .. })
        ));
    }
}
