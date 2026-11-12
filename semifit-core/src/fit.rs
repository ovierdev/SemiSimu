// Resultado de una regresion lineal y = slope * x + intercept
#[derive(Debug, Clone, Copy)]
pub struct LinearFit {
    pub slope: f64,
    pub intercept: f64,
    pub r2: f64,
}

// Minimos cuadrado ordinarios
// Devuelve Non si hay menos de 2 puntos o si todos los x son iguales
pub fn linear_regression(x: &[f64], y: &[f64]) -> Option<LinearFit> {
    assert_eq!(x.len(), y.len(), "x e y deben tener la misma longitud");
    let n = x.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mx = x.iter().sum::<f64>() / nf;
    let my = y.iter().sum::<f64>() / nf;

    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for (&xi, &yi) in x.iter().zip(y) {
        let dx = xi - mx;
        let dy = yi - my;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }

    if sxx == 0.0 {
        return None;
    }

    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let r2 = if syy == 0.0 {
        1.0
    } else {
        sxy * sxy / (sxx * syy)
    };
    Some(LinearFit {
        slope,
        intercept,
        r2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recta_exacta() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let y: Vec<f64> = x.iter().map(|&xi| 3.0 * xi + 2.0).collect();
        let f = linear_regression(&x, &y).unwrap();
        assert!((f.slope - 3.0).abs() < 1e-12);
        assert!((f.intercept - 2.0).abs() < 1e-12);
        assert!((f.r2 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn un_solo_punto_no_se_ajusta() {
        assert!(linear_regression(&[1.0], &[2.0]).is_none());
    }
}
