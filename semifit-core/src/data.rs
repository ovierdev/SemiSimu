use std::fmt;
use std::fs;
use std::path::Path;

// Curva I-V medida: voltajes [V] y corrientes [A]
#[derive(Debug, Clone)]
pub struct IvCurve {
    pub v: Vec<f64>,
    pub i: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct CsvLoad {
    pub curve: IvCurve,
    // Filas numericas descartadas por contener Nan o infinito
    pub skipped: usize,
}

// Errores pisbles al cargar los datos
#[derive(Debug)]
pub enum DataError {
    Io(std::io::Error),
    Parse {
        line: usize,
        content: String,
    },
    MissingColumn {
        line: usize,
        col: usize,
        found: usize,
    },
    Empty,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(e) => write!(f, "error de lectura: {e}"),
            DataError::Parse { line, content } => {
                write!(f, "linea {line} invalida: \"{content}\"")
            }
            DataError::MissingColumn { line, col, found } => write!(
                f,
                " linea {line}: se pidio la columna {} pero solo hay {found}",
                col + 1
            ),
            DataError::Empty => write!(f, "el archivo no contiene datos"),
        }
    }
}

impl std::error::Error for DataError {}

// Permite usar `?` sobre errores de E/S dentro de funciones que devuelven DataError
impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl IvCurve {
    // interpreta text CSV con columnas I-V.
    // Ignora lineas vacia, comentarios (#) y encabezados antes del primer dato.
    pub fn from_csv_columns(text: &str, v_col: usize, i_col: usize) -> Result<CsvLoad, DataError> {
        let mut v = Vec::new();
        let mut i = Vec::new();
        let mut skipped = 0;
        let mut filas_numericas = 0;
        let necesarias = v_col.max(i_col) + 1;

        for (idx, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            if cols.len() < necesarias {
                return Err(DataError::MissingColumn {
                    line: idx + 1,
                    col: necesarias - 1,
                    found: cols.len(),
                });
            }

            match (cols[v_col].parse::<f64>(), cols[i_col].parse::<f64>()) {
                (Ok(x), Ok(y)) => {
                    filas_numericas += 1;
                    if x.is_finite() && y.is_finite() {
                        v.push(x);
                        i.push(y);
                    } else {
                        skipped += 1;
                    }
                }
                _ if filas_numericas == 0 => continue,
                _ => {
                    return Err(DataError::Parse {
                        line: idx + 1,
                        content: line.to_string(),
                    });
                }
            }
        }

        if v.is_empty() {
            return Err(DataError::Empty);
        }
        Ok(CsvLoad {
            curve: Self { v, i },
            skipped,
        })
    }

    pub fn from_csv_str(text: &str) -> Result<Self, DataError> {
        Self::from_csv_columns(text, 0, 1).map(|carga| carga.curve)
    }

    pub fn from_csv_file(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let text = fs::read_to_string(path)?;
        Self::from_csv_str(&text)
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }
}

pub fn csv_headers(text: &str) -> Vec<String> {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        let numerica = cols.iter().all(|c| c.parse::<f64>().is_ok());
        return if numerica {
            (1..=cols.len()).map(|k| format!("columna {k}")).collect()
        } else {
            cols.iter().map(|c| c.to_string()).collect()
        };
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRES_COLS: &str =
        "v_set,v_meas,i_meas\n0.0,nan,nan\n0.1,0.086,2.2e-5\n0.2,0.186,1.1e-3\n";

    #[test]
    fn elige_columnas_y_descarta_nan() {
        let carga = IvCurve::from_csv_columns(TRES_COLS, 1, 2).unwrap();
        assert_eq!(carga.skipped, 1);
        assert_eq!(carga.curve.len(), 2);
        assert_eq!(carga.curve.v[0], 0.086);
        assert_eq!(carga.curve.i[1], 1.1e-3);
    }

    #[test]
    fn columna_inexistente() {
        assert!(matches!(
            IvCurve::from_csv_columns(TRES_COLS, 1, 5),
            Err(DataError::MissingColumn {
                col: 5,
                found: 3,
                ..
            })
        ));
    }

    #[test]
    fn encabezados_con_y_sin_nombre() {
        assert_eq!(csv_headers(TRES_COLS), ["v_set", "v_meas", "i_meas"]);
        assert_eq!(csv_headers("# x\n0.1,1e-9\n"), ["columna 1", "columna 2"]);
    }

    #[test]
    fn lee_con_comentario_y_encabezado() {
        let txt = "# comentario\nV,I\n0.1, 1e-12\n0.2,2e-11\n";
        let c = IvCurve::from_csv_str(txt).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.v[1], 0.2);
        assert_eq!(c.i[0], 1e-12);
    }

    #[test]
    fn reporta_linea_invalida() {
        let txt = "V,I\n0.1,1e-12\n0.2,abc\n";
        match IvCurve::from_csv_str(txt) {
            Err(DataError::Parse { line, .. }) => assert_eq!(line, 3),
            otro => panic!("se esperaba error de parseo, llegó {otro:?}"),
        }
    }

    #[test]
    fn archivo_sin_datos() {
        assert!(matches!(
            IvCurve::from_csv_str("V,I\n"),
            Err(DataError::Empty)
        ));
    }
}
