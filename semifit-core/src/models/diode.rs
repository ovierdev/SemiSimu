use crate::constants::thermal_voltage;

// Parametros del diodo ideal de Schockley
#[derive(Debug, Clone, Copy)]
pub struct DiodeParams {
    // Corriente de saturacion [A]
    pub is: f64,
    // Factor de idealidad [adimensional]
    pub n: f64,
}

// Corriente del diodo [A] para un voltaje `v` [V] y temperatura `t` [K]
pub fn current(p: &DiodeParams, v: f64, t: f64) -> f64 {
    let vt = thermal_voltage(t);
    p.is * (v / (p.n * vt)).exp_m1()
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: DiodeParams = DiodeParams { is: 1e-14, n: 1.0 };

    #[test]
    fn corriente_cero_en_cero_volts() {
        assert_eq!(current(&P, 0.0, 300.0), 0.0);
    }

    #[test]
    fn inversa_satura_en_menos_is() {
        let i = current(&P, -1.0, 300.0);
        assert!((i + P.is).abs() < 1e-20);
    }

    #[test]
    fn pendiente_de_una_decada_por_60mv() {
        // Con n = 1 a 300 K, la corriente sube 10x cada ln(10)·V_T ≈ 59.5 mV
        let dv = 10f64.ln() * thermal_voltage(300.0);
        let ratio = current(&P, 0.5 + dv, 300.0) / current(&P, 0.5, 300.0);
        assert!((ratio - 10.0).abs() < 1e-6);
    }
}
