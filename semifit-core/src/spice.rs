use crate::models::diode::DiodeParams;

// Genera una tarjeta .model de SPICE para un diodo
pub fn diode_model_card(name: &str, p: &DiodeParams) -> String {
    format!(".model {name} D(IS={:.4e} N={:.4})", p.is, p.n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formato_de_tarjeta() {
        let p = DiodeParams {
            is: 2.5e-14,
            n: 1.08,
        };
        assert_eq!(
            diode_model_card("D1", &p),
            ".model D1 D(IS=2.5000e-14 N=1.0800)"
        );
    }
}
