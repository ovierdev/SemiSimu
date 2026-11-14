use crate::constants::thermal_voltage;
use crate::data::IvCurve;
use crate::fit::linear_regression;
use crate::models::diode::DiodeParams;
use std::fmt;

//---- warning thresholds ----
const N_MIN: f64 = 0.8;
const N_MAX: f64 = 5.0;
const T_MIN: f64 = 20.0;
const T_MAX: f64 = 1000.0;
const RMS_MAX_PCT: f64 = 10.0;
const COMPLIANCE_REL_TOL: f64 = 0.01;
const COMPLIANCE_MIN_POINTS: usize = 3;

// ---Errores: el ajuste no se pudo hacer---
#[derive(Debug)]
pub enum ExtractError {
    TooFewPoints {
        found: usize,
        nonpositive: usize,
        compilance: usize,
    },
    Degenerate,
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::TooFewPoints {
                found,
                nonpositive,
                compilance,
            } => write!(
                f,
                "solo {found} puntos utiles en la ventana ({nonpositive} con corriente <= 0,
                {compilance} en el limite del equipo); se necesitan al menos 3"
            ),
            ExtractError::Degenerate => write!(f, "los datos no forman una exponecial creciente"),
        }
    }
}

impl std::error::Error for ExtractError {}

//--- Advertencias: el ajuste se hizo, pero algo que revisar ---
#[derive(Debug, Clone, PartialEq)]
pub enum Warning {
    Compliance { count: usize, current: f64 },
    NonPositiveSkipped { count: usize },
    TemperatureOutOfRange { t: f64 },
    IdealityOutOfRange { n: f64 },
    HighRmsError { pct: f64 },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::Compliance { count, current } => write!(
                f,
                "{count} puntos en el limite de corriente del equipo({current:.4e} A); \
                se excluyeron del ajuste"
            ),
            Warning::NonPositiveSkipped { count } => write!(
                f,
                "{count} puntos con corriente <= 0 en la ventana (ruido u offset del equipo); se omitieron"
            ),
            Warning::TemperatureOutOfRange { t } => write!(
                f,
                "T = {t} K esta fuera del rango {T_MIN} - {T_MAX} L; La escribiste en kelvin?"
            ),
            Warning::IdealityOutOfRange { n } => write!(
                f,
                "n = {n:.3} está fuera del rango físico {N_MIN}–{N_MAX}; revisa la temperatura y la ventana"
            ),
            Warning::HighRmsError { pct } => write!(
                f,
                "error RMS {pct:.1} %: el modelo no describe bien estos datos; \
                prueba una ventana mas angosta"
            ),
        }
    }
}

// Resultado de la extraccion, con metricas de calidad y advertencias
#[derive(Debug, Clone)]
pub struct DiodeExtraction {
    pub params: DiodeParams,
    pub r2: f64,
    // Error RMS relativo entre el modelo y medicion [%]
    pub rms_pct: f64,
    pub points_used: usize,
    pub warnings: Vec<Warning>,
}

// Marca  los puntos en el limite de corriente (compliance) del equipo:
// si al menos 3 puntos estan a menos de 1% de la corriente maxima, son compliance.
pub fn compliance_mask(i: &[f64]) -> Vec<bool> {
    let i_max = i
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .fold(f64::NEG_INFINITY, f64::max);
    if i_max <= 0.0 {
        return vec![false; i.len()];
    }

    let mask: Vec<bool> = i
        .iter()
        .map(|&x| (i_max - x).abs() <= COMPLIANCE_REL_TOL * i_max)
        .collect();

    if mask.iter().filter(|&&m| m).count() >= COMPLIANCE_MIN_POINTS {
        mask
    } else {
        vec![false; i.len()]
    }
}

// Extraer Is y n ajustando ln(I) vs V dentro de la ventana [v_min, v_max].
// Omite ( y reporta) puntos de compliance y corrientes <= 0.
pub fn diode_log_fit(
    curve: &IvCurve,
    t: f64,
    v_min: f64,
    v_max: f64,
) -> Result<DiodeExtraction, ExtractError> {
    let compliance = compliance_mask(&curve.i);

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let (mut n_compliance, mut n_nonpos) = (0, 0);
    let mut i_compliance = 0.0;

    for ((&v, &i), &en_limite) in curve.v.iter().zip(&curve.i).zip(&compliance) {
        if v < v_min || v > v_max {
            continue;
        }
        if en_limite {
            n_compliance += 1;
            i_compliance = i;
            continue;
        }
        if i <= 0.0 {
            n_nonpos += 1;
            continue;
        }
        xs.push(v);
        ys.push(i.ln());
    }

    if xs.len() < 3 {
        return Err(ExtractError::TooFewPoints {
            found: xs.len(),
            nonpositive: n_nonpos,
            compilance: n_compliance,
        });
    }

    let fit = linear_regression(&xs, &ys).ok_or(ExtractError::Degenerate)?;
    if fit.slope <= 0.0 {
        return Err(ExtractError::Degenerate);
    }

    // Erro RMS en ln(I), convertido a porcentaje relativo de corriente
    let ss: f64 = xs
        .iter()
        .zip(&ys)
        .map(|(&x, &y)| {
            let e = y - (fit.slope * x + fit.intercept);
            e * e
        })
        .sum();

    let rms_pct = 100.0 * (ss / xs.len() as f64).sqrt().exp_m1();

    let vt = thermal_voltage(t);
    let params = DiodeParams {
        is: fit.intercept.exp(),
        n: 1.0 / (fit.slope * vt),
    };

    let mut warnings = Vec::new();
    if n_compliance > 0 {
        warnings.push(Warning::Compliance {
            count: n_compliance,
            current: i_compliance,
        });
    }
    if n_nonpos > 0 {
        warnings.push(Warning::NonPositiveSkipped { count: n_nonpos });
    }
    if !(T_MIN..=T_MAX).contains(&t) {
        warnings.push(Warning::TemperatureOutOfRange { t });
    }
    if !(N_MIN..=N_MAX).contains(&params.n) {
        warnings.push(Warning::IdealityOutOfRange { n: params.n });
    }
    if rms_pct > RMS_MAX_PCT {
        warnings.push(Warning::HighRmsError { pct: rms_pct });
    }

    Ok(DiodeExtraction {
        params,
        r2: fit.r2,
        rms_pct,
        points_used: xs.len(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::diode::current;

    const REAL: DiodeParams = DiodeParams {
        is: 2.5e-14,
        n: 1.08,
    };

    fn curva_ideal(p: &DiodeParams, t: f64) -> IvCurve {
        let v: Vec<f64> = (0..=80).map(|k| k as f64 * 0.01).collect();
        let i = v.iter().map(|&x| current(p, x, t)).collect();
        IvCurve { v, i }
    }

    fn tiene(r: &DiodeExtraction, pred: impl Fn(&Warning) -> bool) -> bool {
        r.warnings.iter().any(pred)
    }

    #[test]
    fn recupera_parametros_sin_ruido_y_sin_avisos() {
        let r = diode_log_fit(&curva_ideal(&REAL, 300.0), 300.0, 0.3, 0.7).unwrap();
        assert!((r.params.n / REAL.n - 1.0).abs() < 1e-3);
        assert!((r.params.is / REAL.is - 1.0).abs() < 1e-3);
        assert!(r.rms_pct < 0.1);
        assert!(
            r.warnings.is_empty(),
            "avisos inesperados: {:?}",
            r.warnings
        );
    }

    #[test]
    fn omite_corriente_cero_con_aviso() {
        // La ventana incluye V = 0 → I = 0
        let r = diode_log_fit(&curva_ideal(&REAL, 300.0), 300.0, 0.0, 0.5).unwrap();
        assert!(tiene(&r, |w| matches!(
            w,
            Warning::NonPositiveSkipped { count: 1 }
        )));
    }

    #[test]
    fn detecta_y_excluye_compliance() {
        let mut c = curva_ideal(&REAL, 300.0);
        for i in c.i.iter_mut() {
            *i = i.min(1e-3); // el "equipo" no deja pasar más de 1 mA
        }
        let r = diode_log_fit(&c, 300.0, 0.3, 0.8).unwrap();
        assert!(tiene(&r, |w| matches!(w, Warning::Compliance { .. })));
        assert!((r.params.n / REAL.n - 1.0).abs() < 1e-3); // la pila no contaminó el ajuste
    }

    #[test]
    fn avisa_temperatura_absurda() {
        let r = diode_log_fit(&curva_ideal(&REAL, 300.0), 0.7, 0.3, 0.7).unwrap();
        assert!(tiene(&r, |w| matches!(
            w,
            Warning::TemperatureOutOfRange { .. }
        )));
        assert!(tiene(&r, |w| matches!(
            w,
            Warning::IdealityOutOfRange { .. }
        )));
    }

    #[test]
    fn solo_corrientes_negativas_es_error() {
        let v: Vec<f64> = (0..10).map(|k| k as f64 * 0.1).collect();
        let i = vec![-4e-10; 10];
        match diode_log_fit(&IvCurve { v, i }, 300.0, 0.0, 1.0) {
            Err(ExtractError::TooFewPoints {
                found: 0,
                nonpositive: 10,
                ..
            }) => {}
            otro => panic!("se esperaba TooFewPoints, llegó {otro:?}"),
        }
    }
}
