use semifit_core::data::IvCurve;
use semifit_core::extract::diode_log_fit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curva = IvCurve::from_csv_file("datos/diodo_sintetico.csv")?;
    let r = diode_log_fit(&curva, 300.0, 0.3, 0.7)?;

    let (is_real, n_real) = (2.5e-14, 1.08);
    println!("Puntos usados: {}", r.points_used);
    println!("R²           : {:.6}", r.r2);
    println!(
        "Is = {:.4e} A  (real {:.4e}, error {:+.2} %)",
        r.params.is,
        is_real,
        100.0 * (r.params.is / is_real - 1.0)
    );
    println!(
        "n  = {:.4}       (real {:.4}, error {:+.2} %)",
        r.params.n,
        n_real,
        100.0 * (r.params.n / n_real - 1.0)
    );
    Ok(())
}
