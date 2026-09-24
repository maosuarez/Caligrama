//! Animaciones: varios fotogramas del mismo caligrama, su exportación a SVG
//! animado y el color para la terminal. Sin Python, como `core`.

use std::fmt::Write as _;

use image::DynamicImage;
use unicode_segmentation::UnicodeSegmentation;

use crate::core::{componer, Error, Opciones, Silueta, Texto};

/// Tope de fotogramas cuando se calculan solos.
pub const FOTOGRAMAS_MAX: usize = 240;
/// Fotogramas por defecto cuando el texto no se mueve (solo late).
const FOTOGRAMAS_LATIDO: usize = 24;

#[derive(Debug, Clone)]
pub struct Animacion {
    /// `None`: los justos para que el texto dé la vuelta completa y el bucle
    /// no tenga costura (hasta `FOTOGRAMAS_MAX`).
    pub fotogramas: Option<usize>,
    /// Letras que avanza el texto en cada fotograma (0 = quieto).
    pub paso: usize,
    /// Cuánto se encoge la figura entre latidos, de 0 (nada) a <1. La figura
    /// late una vez por vuelta de la animación.
    pub latido: f32,
}

impl Default for Animacion {
    fn default() -> Self {
        Self {
            fotogramas: None,
            paso: 1,
            latido: 0.0,
        }
    }
}

/// Pulso doble (lub-dub) en [0, 1] para `t` en [0, 1).
fn pulso(t: f32) -> f32 {
    let lub = (-((t - 0.10) / 0.06).powi(2)).exp();
    let dub = 0.6 * (-((t - 0.32) / 0.07).powi(2)).exp();
    lub.max(dub)
}

fn mcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        mcd(b, a % b)
    }
}

/// Fotogramas del caligrama animado, todos con el mismo número de filas y
/// alineados entre sí.
pub fn animar(
    img: &DynamicImage,
    texto: &str,
    op: &Opciones,
    an: &Animacion,
) -> Result<Vec<String>, Error> {
    if !(0.0..1.0).contains(&an.latido) {
        return Err(Error::OpcionInvalida("latido debe estar entre 0 y 1"));
    }
    if an.fotogramas == Some(0) {
        return Err(Error::OpcionInvalida("fotogramas debe ser > 0"));
    }
    let silueta = Silueta::nueva(img, op)?;
    let texto = Texto::nuevo(texto, op.espacios, op.repetir)?;
    let columnas = silueta.columnas(op.ancho, texto.letras)?;

    let n = an.fotogramas.unwrap_or_else(|| match an.paso {
        0 if an.latido > 0.0 => FOTOGRAMAS_LATIDO,
        0 => 1,
        p => (texto.ciclo() / mcd(texto.ciclo(), p)).min(FOTOGRAMAS_MAX),
    });

    let rejillas: Vec<_> = (0..n)
        .map(|i| {
            let escala = 1.0 - an.latido * (1.0 - pulso(i as f32 / n as f32));
            silueta.rejilla_escalada(columnas, escala)
        })
        .collect();
    let desfases: Vec<usize> = (0..n).map(|i| op.desfase + i * an.paso).collect();
    // `componer` recorta todos con los mismos márgenes: mismo número de filas.
    componer(&rejillas, &texto, &desfases)
}

/// Color `#rgb` o `#rrggbb`.
pub fn color(s: &str) -> Result<[u8; 3], Error> {
    const MAL: Error = Error::OpcionInvalida("los colores van en hexadecimal: #rgb o #rrggbb");
    let hex = s.trim().strip_prefix('#').ok_or(MAL)?;
    let bytes: Vec<u8> = match hex.len() {
        3 => hex
            .chars()
            .map(|c| c.to_digit(16).map(|d| d as u8 * 17))
            .collect::<Option<_>>()
            .ok_or(MAL)?,
        6 => (0..3)
            .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
            .collect::<Option<_>>()
            .ok_or(MAL)?,
        _ => return Err(MAL),
    };
    Ok([bytes[0], bytes[1], bytes[2]])
}

/// Color en la posición `t` ∈ [0, 1] de un degradado.
fn mezclar(colores: &[[u8; 3]], t: f32) -> [u8; 3] {
    if colores.len() == 1 {
        return colores[0];
    }
    let pos = t.clamp(0.0, 1.0) * (colores.len() - 1) as f32;
    let i = (pos as usize).min(colores.len() - 2);
    let f = pos - i as f32;
    let (a, b) = (colores[i], colores[i + 1]);
    [0, 1, 2].map(|k| (a[k] as f32 + (b[k] as f32 - a[k] as f32) * f).round() as u8)
}

/// Pinta un fotograma con un degradado horizontal (ANSI truecolor).
pub fn colorear(fotograma: &str, colores: &[[u8; 3]]) -> String {
    if colores.is_empty() {
        return fotograma.to_string();
    }
    let ancho = fotograma
        .lines()
        .map(|l| l.graphemes(true).count())
        .max()
        .unwrap_or(1)
        .max(2);
    let mut s = String::new();
    for (j, linea) in fotograma.split('\n').enumerate() {
        if j > 0 {
            s.push('\n');
        }
        let mut previo = None;
        for (c, g) in linea.graphemes(true).enumerate() {
            let [r, v, a] = mezclar(colores, c as f32 / (ancho - 1) as f32);
            if g != " " && previo != Some([r, v, a]) {
                let _ = write!(s, "\x1b[38;2;{r};{v};{a}m");
                previo = Some([r, v, a]);
            }
            s.push_str(g);
        }
        if previo.is_some() {
            s.push_str("\x1b[0m");
        }
    }
    s
}

#[derive(Debug, Clone)]
pub struct EstiloSvg {
    /// Segundos por fotograma.
    pub intervalo: f32,
    /// Degradado horizontal (uno o más colores hexadecimales).
    pub colores: Vec<[u8; 3]>,
    /// Color de fondo; `None` = transparente.
    pub fondo: Option<[u8; 3]>,
    /// Tamaño de letra en px.
    pub tamano: f32,
    /// Alto/ancho de un carácter, como en `Opciones`.
    pub aspecto: f32,
}

impl Default for EstiloSvg {
    fn default() -> Self {
        Self {
            intervalo: 0.08,
            colores: vec![[0xff, 0x2d, 0x55], [0xb4, 0x4d, 0xff]],
            fondo: None,
            tamano: 14.0,
            aspecto: 2.0,
        }
    }
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn escapar(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// SVG con los fotogramas en bucle. Cada palabra lleva su `x` exacta, así que
/// no depende de que el visor respete los espacios. Los visores sin CSS
/// muestran el primer fotograma.
pub fn a_svg(fotogramas: &[String], e: &EstiloSvg) -> Result<String, Error> {
    if fotogramas.is_empty() {
        return Err(Error::OpcionInvalida("no hay fotogramas"));
    }
    if e.colores.is_empty() {
        return Err(Error::OpcionInvalida("hace falta al menos un color"));
    }
    if !(e.intervalo > 0.0 && e.tamano > 0.0 && e.aspecto > 0.0) {
        return Err(Error::OpcionInvalida(
            "intervalo, tamaño y aspecto deben ser > 0",
        ));
    }
    let cols = fotogramas
        .iter()
        .flat_map(|f| f.split('\n'))
        .map(|l| l.graphemes(true).count())
        .max()
        .unwrap_or(0);
    let filas = fotogramas
        .iter()
        .map(|f| f.split('\n').count())
        .max()
        .unwrap_or(0);
    let cw = e.tamano * 0.6; // avance de una letra monoespaciada
    let lh = cw * e.aspecto;
    let pad = if e.fondo.is_some() {
        e.tamano * 1.5
    } else {
        0.0
    };
    let (w, h) = (
        (cols as f32 * cw + 2.0 * pad).ceil(),
        (filas as f32 * lh + 2.0 * pad).ceil(),
    );

    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="caligrama">"#
    );
    let relleno = if e.colores.len() == 1 {
        hex(e.colores[0])
    } else {
        s.push_str(r#"<defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="0">"#);
        let n = e.colores.len() - 1;
        for (i, c) in e.colores.iter().enumerate() {
            let _ = write!(
                s,
                r#"<stop offset="{:.3}" stop-color="{}"/>"#,
                i as f32 / n as f32,
                hex(*c)
            );
        }
        s.push_str("</linearGradient></defs>");
        "url(#g)".into()
    };
    let _ = write!(
        s,
        "<style>text{{font-family:'DejaVu Sans Mono',Menlo,Consolas,'Liberation Mono',monospace;font-size:{}px;font-weight:bold;fill:{relleno}}}",
        e.tamano
    );
    let n = fotogramas.len();
    if n > 1 {
        let dur = n as f32 * e.intervalo;
        let _ = write!(
            s,
            ".f{{opacity:0;animation:v {dur:.3}s step-end infinite}}@keyframes v{{0%{{opacity:1}}{:.4}%{{opacity:0}}100%{{opacity:0}}}}",
            100.0 / n as f32
        );
    }
    s.push_str("</style>");
    if let Some(f) = e.fondo {
        let _ = write!(
            s,
            r#"<rect width="100%" height="100%" rx="12" fill="{}"/>"#,
            hex(f)
        );
    }

    for (i, foto) in fotogramas.iter().enumerate() {
        if n > 1 {
            let _ = write!(
                s,
                r#"<g class="f" style="animation-delay:{:.3}s"{}>"#,
                i as f32 * e.intervalo,
                if i > 0 { r#" opacity="0""# } else { "" }
            );
        } else {
            s.push_str("<g>");
        }
        for (j, linea) in foto.split('\n').enumerate() {
            let mut tramos = String::new();
            let mut palabra = String::new();
            let mut inicio = 0;
            for (c, g) in linea
                .graphemes(true)
                .chain(std::iter::once(" "))
                .enumerate()
            {
                if g.trim().is_empty() {
                    if !palabra.is_empty() {
                        let _ = write!(
                            tramos,
                            r#"<tspan x="{:.1}">{}</tspan>"#,
                            pad + inicio as f32 * cw,
                            escapar(&palabra)
                        );
                        palabra.clear();
                    }
                } else {
                    if palabra.is_empty() {
                        inicio = c;
                    }
                    palabra.push_str(g);
                }
            }
            if !tramos.is_empty() {
                // Línea base centrada en la celda (~0.35 em bajo su mitad).
                let y = pad + (j as f32 + 0.5) * lh + 0.35 * e.tamano;
                let _ = write!(s, r#"<text y="{y:.1}">{tramos}</text>"#);
            }
        }
        s.push_str("</g>");
    }
    s.push_str("</svg>");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Ancho;
    use image::{Rgba, RgbaImage};

    fn disco() -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_fn(200, 200, |x, y| {
            let d = ((x as f32 - 100.0).powi(2) + (y as f32 - 100.0).powi(2)).sqrt();
            if d < 80.0 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        }))
    }

    fn op() -> Opciones {
        Opciones {
            ancho: Ancho::Fijo(30),
            ..Default::default()
        }
    }

    #[test]
    fn fluye_una_letra_por_fotograma_y_cierra_el_bucle() {
        let an = Animacion::default();
        let f = animar(&disco(), "abc", &op(), &an).unwrap();
        // "abc" + separador = ciclo de 4: 4 fotogramas sin costura.
        assert_eq!(f.len(), 4);
        let primera = |s: &str| s.trim_start().chars().next().unwrap();
        assert_eq!(primera(&f[0]), 'a');
        assert_eq!(primera(&f[1]), 'b');
        assert_eq!(primera(&f[2]), 'c');
        // El fotograma siguiente al último sería otra vez el primero.
        let o = Opciones { desfase: 4, ..op() };
        assert_eq!(crate::core::dibujar(&disco(), "abc", &o).unwrap(), f[0]);
    }

    #[test]
    fn paso_y_ciclo_usan_el_mcd() {
        let an = Animacion {
            paso: 2,
            ..Default::default()
        };
        // ciclo 6 ("abcde" + separador), paso 2 -> 3 fotogramas.
        assert_eq!(animar(&disco(), "abcde", &op(), &an).unwrap().len(), 3);
    }

    #[test]
    fn late_y_todos_los_fotogramas_miden_igual() {
        let an = Animacion {
            fotogramas: Some(20),
            paso: 0,
            latido: 0.4,
        };
        let f = animar(&disco(), "x", &op(), &an).unwrap();
        let filas: Vec<usize> = f.iter().map(|s| s.split('\n').count()).collect();
        assert!(filas.iter().all(|&n| n == filas[0]));
        let letras: Vec<usize> = f.iter().map(|s| s.matches('x').count()).collect();
        // Pico de latido (t≈0.1) más grande que el reposo (t≈0.7).
        assert!(letras[2] > letras[14], "{letras:?}");
    }

    #[test]
    fn opciones_invalidas() {
        let mal = |an: Animacion| animar(&disco(), "x", &op(), &an).is_err();
        assert!(mal(Animacion {
            latido: 1.0,
            ..Default::default()
        }));
        assert!(mal(Animacion {
            fotogramas: Some(0),
            ..Default::default()
        }));
    }

    #[test]
    fn colores_hex() {
        assert_eq!(color("#fff").unwrap(), [255, 255, 255]);
        assert_eq!(color("#ff2d55").unwrap(), [255, 45, 85]);
        assert!(color("red").is_err());
        assert!(color("#12345").is_err());
        assert!(color("#gg0000").is_err());
    }

    #[test]
    fn colorear_pone_degradado_y_resetea() {
        let s = colorear("ab\n c", &[[255, 0, 0], [0, 0, 255]]);
        assert!(s.starts_with("\x1b[38;2;255;0;0ma"));
        assert!(s.contains("\x1b[38;2;0;0;255m"));
        assert_eq!(s.matches("\x1b[0m").count(), 2);
        assert_eq!(colorear("ab", &[]), "ab");
    }

    #[test]
    fn svg_un_grupo_por_fotograma_y_escapa() {
        let f = vec!["<a> & b".to_string(), "  ñú".to_string()];
        let svg = a_svg(&f, &EstiloSvg::default()).unwrap();
        assert_eq!(svg.matches("<g class=\"f\"").count(), 2);
        assert!(svg.contains("&lt;a&gt;") && svg.contains("&amp;"));
        // "ñú" empieza en la columna 2 aunque "ñ" ocupe 2 bytes.
        assert!(svg.contains(&format!(r#"<tspan x="{:.1}">ñú</tspan>"#, 2.0 * 14.0 * 0.6)));
        assert!(svg.contains("@keyframes"));
        let fijo = a_svg(&f[..1], &EstiloSvg::default()).unwrap();
        assert!(!fijo.contains("@keyframes"));
    }
}
