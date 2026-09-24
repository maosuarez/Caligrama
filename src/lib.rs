//! Binding PyO3: expone `caligrama.dibujar`, `caligrama.analizar` y el CLI.

pub mod core;

use std::fmt::Write as _;
use std::path::PathBuf;

use image::DynamicImage;
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::core::{Analisis, Ancho, Error, Opciones, Silueta};

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::Imagen(image::ImageError::IoError(io)) => PyOSError::new_err(io.to_string()),
            otro => PyValueError::new_err(otro.to_string()),
        }
    }
}

/// `bytes` va primero: la extracción de PathBuf también acepta bytes.
#[derive(FromPyObject)]
enum Imagen {
    Bytes(Vec<u8>),
    Ruta(PathBuf),
}

impl Imagen {
    fn cargar(&self) -> Result<DynamicImage, Error> {
        Ok(match self {
            Imagen::Bytes(b) => image::load_from_memory(b)?,
            Imagen::Ruta(r) => image::open(r)?,
        })
    }
}

/// `ancho` en Python: entero o `"auto"`.
#[derive(FromPyObject)]
enum AnchoPy {
    Num(usize),
    Texto(String),
}

impl TryFrom<AnchoPy> for Ancho {
    type Error = Error;

    fn try_from(a: AnchoPy) -> Result<Self, Error> {
        match a {
            AnchoPy::Num(n) => Ok(Ancho::Fijo(n)),
            AnchoPy::Texto(s) => s.parse(),
        }
    }
}

/// Escribe `texto` dentro de la silueta de `imagen` (ruta o bytes) y devuelve
/// el resultado como texto multilínea. `ancho="auto"` elige el menor ancho en
/// el que cabe todo el texto.
#[pyfunction]
#[pyo3(signature = (imagen, texto, ancho=AnchoPy::Num(60), aspecto=2.0, repetir=true, espacios="normal", umbral=None, invertir=false, huecos=false, suavizar=0))]
#[allow(clippy::too_many_arguments)] // espejo de los kwargs de Python
fn dibujar(
    imagen: Imagen,
    texto: &str,
    ancho: AnchoPy,
    aspecto: f32,
    repetir: bool,
    espacios: &str,
    umbral: Option<u8>,
    invertir: bool,
    huecos: bool,
    suavizar: usize,
) -> PyResult<String> {
    let op = Opciones {
        ancho: ancho.try_into()?,
        aspecto,
        repetir,
        espacios: espacios.parse()?,
        umbral,
        invertir,
        huecos,
        suavizar,
    };
    Ok(core::dibujar(&imagen.cargar()?, texto, &op)?)
}

/// Anchos de referencia que se comparan en el informe.
const ANCHOS_TABLA: [usize; 7] = [30, 40, 50, 60, 80, 100, 120];

/// Todo lo que `analizar` y `caligrama analizar` reportan.
struct Informe {
    analisis: Analisis,
    /// (ancho, filas, letras) para `ANCHOS_TABLA`.
    tabla: Vec<(usize, usize, usize)>,
    /// Letras del texto y ancho en el que cabe (o el error si no cabe nunca).
    texto: Option<(usize, Result<usize, Error>)>,
}

fn informe(img: &DynamicImage, texto: Option<&str>, op: &Opciones) -> Result<Informe, Error> {
    let silueta = Silueta::nueva(img, op)?;
    let texto = texto.map(|t| {
        let letras = core::contar_letras(t, op.espacios);
        (letras, silueta.ancho_para(letras))
    });
    let columnas = match (op.ancho, &texto) {
        (Ancho::Fijo(n), _) => n,
        (Ancho::Auto, Some((_, Ok(n)))) => *n,
        // No cabe: devuelve el error de ajuste en vez de una plantilla gigante.
        (Ancho::Auto, Some((letras, Err(_)))) => silueta.ancho_para(*letras)?,
        (Ancho::Auto, None) => {
            return Err(Error::OpcionInvalida("ancho=\"auto\" necesita un texto"))
        }
    };
    let tabla = ANCHOS_TABLA
        .iter()
        .filter_map(|&n| silueta.analizar(n).ok().map(|a| (n, a.filas, a.letras)))
        .collect();
    Ok(Informe {
        analisis: silueta.analizar(columnas)?,
        tabla,
        texto,
    })
}

/// Examina la silueta sin escribir nada: cuántas letras/palabras caben, cómo
/// se reparten por fila y, si se pasa `texto`, en qué ancho cabe completo.
#[pyfunction]
#[pyo3(signature = (imagen, texto=None, ancho=AnchoPy::Num(60), aspecto=2.0, espacios="normal", umbral=None, invertir=false, huecos=false, suavizar=0))]
#[allow(clippy::too_many_arguments)]
fn analizar<'py>(
    py: Python<'py>,
    imagen: Imagen,
    texto: Option<&str>,
    ancho: AnchoPy,
    aspecto: f32,
    espacios: &str,
    umbral: Option<u8>,
    invertir: bool,
    huecos: bool,
    suavizar: usize,
) -> PyResult<Bound<'py, PyDict>> {
    let op = Opciones {
        ancho: ancho.try_into()?,
        aspecto,
        espacios: espacios.parse()?,
        umbral,
        invertir,
        huecos,
        suavizar,
        ..Default::default()
    };
    let inf = informe(&imagen.cargar()?, texto, &op)?;
    let a = &inf.analisis;
    let d = PyDict::new(py);
    d.set_item("ancho", a.ancho)?;
    d.set_item("filas", a.filas)?;
    d.set_item("letras", a.letras)?;
    d.set_item("palabras_aprox", a.palabras_aprox())?;
    d.set_item("letras_por_fila", &a.por_fila)?;
    d.set_item("tramos", a.tramos)?;
    d.set_item("tramo_min", a.tramo_min)?;
    d.set_item("tramo_max", a.tramo_max)?;
    d.set_item("plantilla", &a.plantilla)?;
    let tabla = PyDict::new(py);
    for &(n, _, letras) in &inf.tabla {
        tabla.set_item(n, letras)?;
    }
    d.set_item("letras_por_ancho", tabla)?;
    if let Some((letras, ajuste)) = &inf.texto {
        d.set_item("texto_letras", letras)?;
        d.set_item("ancho_recomendado", ajuste.as_ref().ok())?;
        d.set_item("sobran", a.letras as isize - *letras as isize)?;
    }
    Ok(d)
}

fn formatear_informe(inf: &Informe) -> String {
    let a = &inf.analisis;
    let mut s = String::new();
    let _ = writeln!(s, "Silueta a {} columnas × {} filas", a.ancho, a.filas);
    let _ = writeln!(
        s,
        "Caben {} letras (los espacios cuentan) ≈ {} palabras",
        a.letras,
        a.palabras_aprox()
    );
    let _ = writeln!(
        s,
        "Tramos: {} (de {} a {} letras); cada hueco entre tramos puede partir una palabra",
        a.tramos, a.tramo_min, a.tramo_max
    );

    let _ = writeln!(s, "\nPlantilla (letras por fila a la derecha):");
    let lineas: Vec<&str> = a.plantilla.lines().collect();
    let largo = lineas.iter().map(|l| l.len()).max().unwrap_or(0);
    for (l, n) in lineas.iter().zip(&a.por_fila) {
        let _ = writeln!(s, "  {l:<largo$}  │ {n:>3}");
    }

    let _ = writeln!(s, "\nOtros anchos:\n  ancho  filas  letras  ≈palabras");
    for &(n, filas, letras) in &inf.tabla {
        let _ = writeln!(s, "  {n:>5}  {filas:>5}  {letras:>6}  {:>9}", letras / 6);
    }

    if let Some((letras, ajuste)) = &inf.texto {
        let _ = write!(s, "\nTu texto: {letras} letras. ");
        match ajuste {
            Ok(n) => {
                let _ = write!(s, "Cabe completo desde {n} columnas (-w auto). ");
            }
            Err(e) => {
                let _ = write!(s, "{e}. ");
            }
        }
        let diferencia = a.letras as isize - *letras as isize;
        let _ = match diferencia {
            0 => writeln!(s, "A {} columnas cabe exacto.", a.ancho),
            d if d > 0 => writeln!(s, "A {} columnas sobran {d} letras.", a.ancho),
            d => writeln!(s, "A {} columnas faltan {} letras.", a.ancho, -d),
        };
    }
    s.trim_end().to_string()
}

const AYUDA: &str = "\
uso: caligrama IMAGEN [opciones]            escribe el texto dentro de la silueta
     caligrama analizar IMAGEN [opciones]   examina la silueta para diseñar el texto

Al dibujar, si no se pasa --texto ni --archivo, el texto se lee de stdin.
Al analizar, el texto es opcional: si se pasa, dice en qué ancho cabe.

opciones:
  -t, --texto TEXTO     mensaje a escribir
  -f, --archivo RUTA    leer el mensaje de un archivo
  -w, --ancho N|auto    columnas de salida (60); auto = el menor ancho donde cabe todo
      --aspecto F       alto/ancho de un carácter (2.0)
      --umbral N        umbral 0-255 de separación figura/fondo (auto)
      --sin-repetir     no repetir el texto para llenar la silueta
      --espacios MODO   normal: colapsa espacios y separa repeticiones con uno (defecto)
                        sin: quita todos los espacios; todos: los deja tal cual
      --invertir        escribir en el fondo en vez de en la figura
      --con-huecos      respetar los huecos interiores del color del fondo
      --suavizar N      unir trazos punteados o hechos de letras (radio en píxeles)
  -h, --help            mostrar esta ayuda";

/// Punto de entrada del script `caligrama`. Devuelve el código de salida.
#[pyfunction]
fn cli(py: Python<'_>) -> PyResult<i32> {
    let argv: Vec<String> = py.import("sys")?.getattr("argv")?.extract()?;
    match ejecutar_cli(&argv[1..]) {
        Ok(salida) => {
            println!("{salida}");
            Ok(0)
        }
        Err(msg) if msg.is_empty() => {
            println!("{AYUDA}");
            Ok(0)
        }
        Err(msg) => {
            eprintln!("caligrama: {msg}\n\n{AYUDA}");
            Ok(2)
        }
    }
}

/// `Err("")` significa que se pidió la ayuda.
fn ejecutar_cli(args: &[String]) -> Result<String, String> {
    let (modo_analizar, args) = match args.first().map(String::as_str) {
        Some("analizar") => (true, &args[1..]),
        _ => (false, args),
    };
    let mut op = Opciones::default();
    let (mut ruta, mut texto, mut archivo) = (None, None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut valor = |nombre: &str| {
            it.next()
                .cloned()
                .ok_or(format!("falta el valor de {nombre}"))
        };
        match a.as_str() {
            "-h" | "--help" => return Err(String::new()),
            "-t" | "--texto" => texto = Some(valor(a)?),
            "-f" | "--archivo" => archivo = Some(valor(a)?),
            "-w" | "--ancho" => op.ancho = valor(a)?.parse().map_err(|e: Error| e.to_string())?,
            "--aspecto" => {
                op.aspecto = valor(a)?
                    .parse()
                    .map_err(|_| "--aspecto debe ser un número")?
            }
            "--umbral" => {
                op.umbral = Some(
                    valor(a)?
                        .parse()
                        .map_err(|_| "--umbral debe estar entre 0 y 255")?,
                )
            }
            "--sin-repetir" => op.repetir = false,
            "--espacios" => op.espacios = valor(a)?.parse().map_err(|e: Error| e.to_string())?,
            "--invertir" => op.invertir = true,
            "--con-huecos" => op.huecos = true,
            "--suavizar" => {
                op.suavizar = valor(a)?
                    .parse()
                    .map_err(|_| "--suavizar debe ser un entero >= 0")?
            }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(format!("opción desconocida: {s}"))
            }
            _ if ruta.is_none() => ruta = Some(PathBuf::from(a)),
            _ => return Err(format!("argumento de más: {a}")),
        }
    }
    let ruta = ruta.ok_or("falta la ruta de la imagen")?;
    let texto = match (texto, archivo) {
        (Some(_), Some(_)) => return Err("usa --texto o --archivo, no ambos".into()),
        (Some(t), None) => Some(t),
        (None, Some(f)) => Some(std::fs::read_to_string(&f).map_err(|e| format!("{f}: {e}"))?),
        (None, None) if modo_analizar => None,
        (None, None) => Some(std::io::read_to_string(std::io::stdin()).map_err(|e| e.to_string())?),
    };
    let img = image::open(&ruta).map_err(|e| Error::from(e).to_string())?;
    if modo_analizar {
        informe(&img, texto.as_deref(), &op)
            .map(|inf| formatear_informe(&inf))
            .map_err(|e| e.to_string())
    } else {
        core::dibujar(&img, texto.as_deref().unwrap_or_default(), &op).map_err(|e| e.to_string())
    }
}

#[pymodule]
fn caligrama(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(dibujar, m)?)?;
    m.add_function(wrap_pyfunction!(analizar, m)?)?;
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
