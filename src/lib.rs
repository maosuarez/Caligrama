//! Binding PyO3: expone `dibujar`, `analizar`, `animar`, `a_svg`,
//! `reproducir` y el CLI `caligrama`.

pub mod animacion;
pub mod core;

use std::fmt::Write as _;
use std::io::{IsTerminal, Write as _};
use std::path::PathBuf;
use std::time::Duration;

use image::DynamicImage;
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::animacion::{Animacion, EstiloSvg};
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
#[pyo3(signature = (imagen, texto, ancho=AnchoPy::Num(60), aspecto=2.0, repetir=true, espacios="normal", umbral=None, invertir=false, huecos=false, suavizar=0, desfase=0))]
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
    desfase: usize,
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
        desfase,
    };
    Ok(core::dibujar(&imagen.cargar()?, texto, &op)?)
}

/// Fotogramas de un caligrama animado: el texto avanza `paso` letras por
/// fotograma y, con `latido` > 0, la figura late una vez por vuelta. Con
/// `fotogramas=None` se calculan los justos para un bucle sin costura.
#[pyfunction]
#[pyo3(signature = (imagen, texto, fotogramas=None, paso=1, latido=0.0, ancho=AnchoPy::Num(60), aspecto=2.0, repetir=true, espacios="normal", umbral=None, invertir=false, huecos=false, suavizar=0, desfase=0))]
#[allow(clippy::too_many_arguments)]
fn animar(
    imagen: Imagen,
    texto: &str,
    fotogramas: Option<usize>,
    paso: usize,
    latido: f32,
    ancho: AnchoPy,
    aspecto: f32,
    repetir: bool,
    espacios: &str,
    umbral: Option<u8>,
    invertir: bool,
    huecos: bool,
    suavizar: usize,
    desfase: usize,
) -> PyResult<Vec<String>> {
    let op = Opciones {
        ancho: ancho.try_into()?,
        aspecto,
        repetir,
        espacios: espacios.parse()?,
        umbral,
        invertir,
        huecos,
        suavizar,
        desfase,
    };
    let an = Animacion {
        fotogramas,
        paso,
        latido,
    };
    Ok(animacion::animar(&imagen.cargar()?, texto, &op, &an)?)
}

/// Uno o varios fotogramas.
#[derive(FromPyObject)]
enum Fotogramas {
    Uno(String),
    Varios(Vec<String>),
}

impl Fotogramas {
    fn lista(self) -> Vec<String> {
        match self {
            Fotogramas::Uno(s) => vec![s],
            Fotogramas::Varios(v) => v,
        }
    }
}

fn colores(lista: &[String]) -> Result<Vec<[u8; 3]>, Error> {
    lista.iter().map(|c| animacion::color(c)).collect()
}

/// SVG (animado si hay varios fotogramas) listo para un README o una web.
#[pyfunction]
#[pyo3(signature = (fotogramas, intervalo=0.08, colores=vec!["#ff2d55".to_string(), "#b44dff".to_string()], fondo=None, tamano=14.0, aspecto=2.0))]
fn a_svg(
    fotogramas: Fotogramas,
    intervalo: f32,
    colores: Vec<String>,
    fondo: Option<String>,
    tamano: f32,
    aspecto: f32,
) -> PyResult<String> {
    let estilo = EstiloSvg {
        intervalo,
        colores: crate::colores(&colores)?,
        fondo: fondo.as_deref().map(animacion::color).transpose()?,
        tamano,
        aspecto,
    };
    Ok(animacion::a_svg(&fotogramas.lista(), &estilo)?)
}

/// Reproduce los fotogramas en la terminal. `veces=None` repite hasta Ctrl+C.
#[pyfunction]
#[pyo3(signature = (fotogramas, intervalo=0.08, veces=None, colores=None))]
fn reproducir(
    py: Python<'_>,
    fotogramas: Fotogramas,
    intervalo: f64,
    veces: Option<usize>,
    colores: Option<Vec<String>>,
) -> PyResult<()> {
    let colores = crate::colores(&colores.unwrap_or_default())?;
    // Rust escribe directo en la salida: lo que Python tenga en su búfer debe
    // salir antes, o se mezclaría con la animación.
    py.import("sys")?.getattr("stdout")?.call_method0("flush")?;
    reproducir_en(&fotogramas.lista(), intervalo, veces, &colores, || {
        py.check_signals()
    })
}

/// Bucle de reproducción: dibuja cada fotograma desde la esquina superior
/// izquierda y borra lo que sobre. `revisar` deja que Python atienda Ctrl+C;
/// pase lo que pase, el cursor se restaura antes de salir.
fn reproducir_en(
    fotogramas: &[String],
    intervalo: f64,
    veces: Option<usize>,
    colores: &[[u8; 3]],
    mut revisar: impl FnMut() -> PyResult<()>,
) -> PyResult<()> {
    if !(intervalo.is_finite() && intervalo >= 0.0) {
        return Err(PyValueError::new_err("intervalo debe ser >= 0"));
    }
    // Todos con el mismo alto para poder volver siempre a la misma fila; cada
    // línea borra lo que quede a su derecha del fotograma anterior.
    let filas = fotogramas
        .iter()
        .map(|f| f.split('\n').count())
        .max()
        .unwrap_or(1);
    let pintados: Vec<String> = fotogramas
        .iter()
        .map(|f| {
            let relleno = "\n".repeat(filas - f.split('\n').count());
            animacion::colorear(&(f.clone() + &relleno), colores).replace('\n', "\x1b[K\n")
                + "\x1b[K"
        })
        .collect();
    // Se redibuja en el sitio (subir `filas - 1` líneas), sin limpiar la
    // pantalla: lo que se imprimió antes sigue visible.
    let subir = if filas > 1 {
        format!("\r\x1b[{}A", filas - 1)
    } else {
        "\r".to_string()
    };
    let io = |e: std::io::Error| PyOSError::new_err(e.to_string());
    let mut out = std::io::stdout().lock();
    write!(out, "\x1b[?25l").map_err(io)?;
    let resultado = (|| {
        let mut primero = true;
        let mut vuelta = 0;
        while veces.is_none_or(|v| vuelta < v) {
            for f in &pintados {
                if !primero {
                    write!(out, "{subir}").map_err(io)?;
                }
                primero = false;
                write!(out, "{f}").map_err(io)?;
                out.flush().map_err(io)?;
                std::thread::sleep(Duration::from_secs_f64(intervalo));
                revisar()?;
            }
            vuelta += 1;
        }
        Ok(())
    })();
    let _ = writeln!(out, "\x1b[0m\x1b[?25h");
    let _ = out.flush();
    resultado
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
     caligrama animar IMAGEN [opciones]     anima el caligrama en la terminal o en un SVG

Si no se pasa --texto ni --archivo, el texto se lee de stdin (al analizar es opcional:
si se pasa, dice en qué ancho cabe).

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
      --desfase N       empezar el texto N letras más adelante
  -h, --help            mostrar esta ayuda

salida (dibujar y animar):
      --colores C1,C2   degradado horizontal en hexadecimal (#ff2d55,#b44dff)
      --svg RUTA        guardar un SVG en vez de imprimir (animado con animar)
      --fondo COLOR     color de fondo del SVG (transparente si no se indica)
      --tamano PX       tamaño de letra del SVG (14)

animar:
      --paso N          letras que avanza el texto por fotograma (1; 0 = quieto)
      --latido F        la figura late encogiéndose hasta F (0-1, p. ej. 0.2)
      --fotogramas N    cuántos fotogramas (auto: los justos para un bucle sin costura)
      --intervalo S     segundos por fotograma (0.08)
      --veces N         vueltas en la terminal (sin fin; 1 si la salida no es una terminal)

Ctrl+C detiene la animación.";

enum Modo {
    Dibujar,
    Analizar,
    Animar,
}

/// Lo que el CLI tiene que hacer al final.
enum Salida {
    Texto(String),
    Reproducir {
        fotogramas: Vec<String>,
        intervalo: f64,
        veces: Option<usize>,
        colores: Vec<[u8; 3]>,
    },
}

/// Punto de entrada del script `caligrama`. Devuelve el código de salida.
#[pyfunction]
fn cli(py: Python<'_>) -> PyResult<i32> {
    let argv: Vec<String> = py.import("sys")?.getattr("argv")?.extract()?;
    match ejecutar_cli(&argv[1..]) {
        Ok(Salida::Texto(t)) => {
            println!("{t}");
            Ok(0)
        }
        Ok(Salida::Reproducir {
            fotogramas,
            intervalo,
            veces,
            colores,
        }) => match reproducir_en(&fotogramas, intervalo, veces, &colores, || {
            py.check_signals()
        }) {
            Ok(()) => Ok(0),
            Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) => Ok(0),
            Err(e) => Err(e),
        },
        Err(msg) if msg.is_empty() => {
            println!("{AYUDA}");
            Ok(0)
        }
        Err(msg) => {
            eprintln!("caligrama: {msg}\n(caligrama --help muestra todas las opciones)");
            Ok(2)
        }
    }
}

fn numero<T: std::str::FromStr>(v: String, msg: &str) -> Result<T, String> {
    v.parse().map_err(|_| msg.to_string())
}

/// `Err("")` significa que se pidió la ayuda.
fn ejecutar_cli(args: &[String]) -> Result<Salida, String> {
    let (modo, args) = match args.first().map(String::as_str) {
        Some("analizar") => (Modo::Analizar, &args[1..]),
        Some("animar") => (Modo::Animar, &args[1..]),
        _ => (Modo::Dibujar, args),
    };
    let texto_err = |e: Error| e.to_string();
    let mut op = Opciones::default();
    let mut an = Animacion::default();
    let mut estilo = EstiloSvg::default();
    let (mut ruta, mut texto, mut archivo) = (None, None, None);
    let (mut svg, mut colores_cli, mut intervalo, mut veces) = (None, None, None, None);
    let (mut de_salida, mut de_animacion) = (None, None);
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
            "-w" | "--ancho" => op.ancho = valor(a)?.parse().map_err(texto_err)?,
            "--aspecto" => op.aspecto = numero(valor(a)?, "--aspecto debe ser un número")?,
            "--umbral" => op.umbral = Some(numero(valor(a)?, "--umbral debe estar entre 0 y 255")?),
            "--sin-repetir" => op.repetir = false,
            "--espacios" => op.espacios = valor(a)?.parse().map_err(texto_err)?,
            "--invertir" => op.invertir = true,
            "--con-huecos" => op.huecos = true,
            "--suavizar" => op.suavizar = numero(valor(a)?, "--suavizar debe ser un entero >= 0")?,
            "--desfase" => op.desfase = numero(valor(a)?, "--desfase debe ser un entero >= 0")?,
            "--svg" => {
                de_salida = Some(a.clone());
                svg = Some(PathBuf::from(valor(a)?));
            }
            "--colores" => {
                de_salida = Some(a.clone());
                colores_cli = Some(valor(a)?);
            }
            "--fondo" => {
                de_salida = Some(a.clone());
                estilo.fondo = Some(animacion::color(&valor(a)?).map_err(texto_err)?);
            }
            "--tamano" => {
                de_salida = Some(a.clone());
                estilo.tamano = numero(valor(a)?, "--tamano debe ser un número")?;
            }
            "--paso" => {
                de_animacion = Some(a.clone());
                an.paso = numero(valor(a)?, "--paso debe ser un entero >= 0")?;
            }
            "--latido" => {
                de_animacion = Some(a.clone());
                an.latido = numero(valor(a)?, "--latido debe ser un número entre 0 y 1")?;
            }
            "--fotogramas" => {
                de_animacion = Some(a.clone());
                an.fotogramas = Some(numero(valor(a)?, "--fotogramas debe ser un entero > 0")?);
            }
            "--intervalo" => {
                de_animacion = Some(a.clone());
                intervalo = Some(numero::<f64>(valor(a)?, "--intervalo debe ser un número")?);
            }
            "--veces" => {
                de_animacion = Some(a.clone());
                veces = Some(numero(valor(a)?, "--veces debe ser un entero >= 0")?);
            }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(format!("opción desconocida: {s}"))
            }
            _ if ruta.is_none() => ruta = Some(PathBuf::from(a)),
            _ => return Err(format!("argumento de más: {a}")),
        }
    }
    match (&modo, de_salida, de_animacion) {
        (Modo::Analizar, Some(o), _) | (Modo::Analizar | Modo::Dibujar, _, Some(o)) => {
            let donde = if matches!(modo, Modo::Analizar) {
                "analizar"
            } else {
                "dibujar"
            };
            return Err(format!("{o} no se usa al {donde}"));
        }
        _ => {}
    }

    let ruta = ruta.ok_or("falta la ruta de la imagen")?;
    let texto = match (texto, archivo) {
        (Some(_), Some(_)) => return Err("usa --texto o --archivo, no ambos".into()),
        (Some(t), None) => Some(t),
        (None, Some(f)) => Some(std::fs::read_to_string(&f).map_err(|e| format!("{f}: {e}"))?),
        (None, None) if matches!(modo, Modo::Analizar) => None,
        (None, None) => Some(std::io::read_to_string(std::io::stdin()).map_err(|e| e.to_string())?),
    };
    let img = image::open(&ruta).map_err(|e| Error::from(e).to_string())?;
    let texto = texto.unwrap_or_default();

    let fotogramas = match modo {
        Modo::Analizar => {
            return informe(
                &img,
                Some(&texto).filter(|t| !t.is_empty()).map(|t| t.as_str()),
                &op,
            )
            .map(|inf| Salida::Texto(formatear_informe(&inf)))
            .map_err(texto_err)
        }
        Modo::Dibujar => vec![core::dibujar(&img, &texto, &op).map_err(texto_err)?],
        Modo::Animar => animacion::animar(&img, &texto, &op, &an).map_err(texto_err)?,
    };
    // En la terminal no hay color salvo que se pida; en el SVG hay degradado por defecto.
    let colores = match &colores_cli {
        Some(c) => c
            .split(',')
            .map(animacion::color)
            .collect::<Result<Vec<_>, _>>()
            .map_err(texto_err)?,
        None => Vec::new(),
    };
    let intervalo = intervalo.unwrap_or(estilo.intervalo as f64);

    if let Some(destino) = svg {
        if !colores.is_empty() {
            estilo.colores = colores;
        }
        estilo.intervalo = intervalo as f32;
        estilo.aspecto = op.aspecto;
        let contenido = animacion::a_svg(&fotogramas, &estilo).map_err(texto_err)?;
        std::fs::write(&destino, contenido).map_err(|e| format!("{}: {e}", destino.display()))?;
        return Ok(Salida::Texto(format!(
            "{}: {} fotograma(s)",
            destino.display(),
            fotogramas.len()
        )));
    }
    match modo {
        Modo::Animar => Ok(Salida::Reproducir {
            fotogramas,
            intervalo,
            // Si la salida va a un archivo o una tubería, no repetir para siempre.
            veces: veces.or((!std::io::stdout().is_terminal()).then_some(1)),
            colores,
        }),
        _ => Ok(Salida::Texto(animacion::colorear(&fotogramas[0], &colores))),
    }
}

#[pymodule]
fn caligrama(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(dibujar, m)?)?;
    m.add_function(wrap_pyfunction!(analizar, m)?)?;
    m.add_function(wrap_pyfunction!(animar, m)?)?;
    m.add_function(wrap_pyfunction!(a_svg, m)?)?;
    m.add_function(wrap_pyfunction!(reproducir, m)?)?;
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
