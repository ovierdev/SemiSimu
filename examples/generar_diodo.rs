use rand_distr::{Distribution, Normal};
use semifit::models::diode::{DiodeParams, current};
use std::fs::{self, File};
use std::io::Write;

fn main() -> std::io::Result<()> {
    // Parametros "verdaderos" que el extractor debera recuperar despues
    let p = DiodeParams {
        is: 2.5e-14,
        n: 1.08,
    };
    let t = 300.0;

    // Ruido relativo del 2% ( los equipos reales tienen error proporcional a la lectura)
    let ruido = Normal::new(0.0, 0.02).unwrap();
    let mut rng = rand::rng();

    fs::create_dir_all("datos")?;
    let mut f = File::create("datos/diodo_sintetico.csv")?;

    writeln!(
        f,
        "# Diodo Sintetico: Is = {:e} A, n = {}, T = {} K",
        p.is, p.n, t
    )?;
    writeln!(f, "V, I")?;

    for k in 0..=70 {
        let v = 0.1 + k as f64 * 0.01; // barrido de 0.1 V a 0.8 V
        let i = current(&p, v, t) * (1.0 + ruido.sample(&mut rng));
        writeln!(f, "{v:.3},{i:.6e}")?;
    }

    println!("Escrito datos/diodo_sintetico.csv");
    Ok(())
}
