use std::fmt;
use std::fs;
use std::path::Path;

// Curva I-V medida: voltajes [V] y corrientes [A]
#[derive(Debug, Clone)]
pub struct IvCurve {
    pub v: Vec<f64>,
    pub i: Vec<f64>,
}

// Errores pisbles al cargar los datos
#[derive(Debug)]
pub enum DataError {
    Io(std::io::Error),
    Parse { line: usize, content: String },
    Empty,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(e) => write!(f, "error de lectura: {e}"),
            DataError::Parse { line, content } => {
                write!(f, "linea {line} invalida: \"{content}\"")
            }
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
    pub fn from_csv_str(text: &str) -> Result<Self, DataError> {
        let mut v = Vec::new();
        let mut i = Vec::new();

        for (idx, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut cols = line.split(',').map(str::trim);
            let (Some(a), Some(b)) = (cols.next(), cols.next()) else {
                return Err(DataError::Parse {
                    line: idx + 1,
                    content: line.to_string(),
                });
            };

            match (a.parse::<f64>(), b.parse::<f64>()) {
                (Ok(x), Ok(y)) => {
                    v.push(x);
                    i.push(y);
                }
                // Si todasvia no hay datos, asumimos que es el encabezado
                _ if v.is_empty() => continue,
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
        Ok(Self { v, i })
    }
    // Lee un archivo CSV del disco
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

#[cfg(test)]
mod tests {
    use super::*;

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
