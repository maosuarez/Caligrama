//! Núcleo puro (sin Python): imagen -> máscara -> rejilla -> texto.

use std::fmt;

use image::{DynamicImage, RgbaImage};
use unicode_segmentation::UnicodeSegmentation;

/// Máximo de columnas que prueba `Ancho::Auto`.
pub const ANCHO_AUTO_MAX: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ancho {
    Fijo(usize),
    /// El menor ancho cuya silueta tiene sitio para todo el texto una vez.
    Auto,
}

/// Cómo se tratan los espacios en blanco del texto.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Espacios {
    /// Colapsa espacios y saltos de línea en uno solo y separa cada
    /// repetición con un espacio.
    #[default]
    Normal,
    /// Quita todos los espacios; las repeticiones van pegadas.
    Sin,
    /// Conserva cada espacio tal cual (un salto de línea o tabulador = un
    /// espacio) y no añade ninguno entre repeticiones.
    Todos,
}

impl std::str::FromStr for Espacios {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Ok(Espacios::Normal),
            "sin" => Ok(Espacios::Sin),
            "todos" => Ok(Espacios::Todos),
            _ => Err(Error::OpcionInvalida(
                "espacios debe ser \"normal\", \"sin\" o \"todos\"",
            )),
        }
    }
}

impl std::str::FromStr for Ancho {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        if s.eq_ignore_ascii_case("auto") {
            return Ok(Ancho::Auto);
        }
        s.parse()
            .map(Ancho::Fijo)
            .map_err(|_| Error::OpcionInvalida("ancho debe ser un entero o \"auto\""))
    }
}

#[derive(Debug, Clone)]
pub struct Opciones {
    /// Columnas de la salida.
    pub ancho: Ancho,
    /// Alto/ancho de una celda de terminal (~2.0).
    pub aspecto: f32,
    /// Repetir el texto hasta llenar la silueta.
    pub repetir: bool,
    pub espacios: Espacios,
    /// Umbral 0-255 de distancia al fondo; `None` = automático (Otsu).
    pub umbral: Option<u8>,
    /// Escribir en el fondo en lugar de en la figura.
    pub invertir: bool,
    /// Respetar los huecos interiores. Por defecto se rellenan: una zona del
    /// color del fondo pero encerrada por la figura (la barriga blanca de un
    /// pingüino sobre fondo blanco) es parte de la silueta.
    pub huecos: bool,
    /// Radio en píxeles de un cierre morfológico (dilatar + erosionar) que une
    /// trazos punteados o hechos de letras. 0 = desactivado.
    pub suavizar: usize,
    /// Letra del texto por la que se empieza a escribir (para hacerlo fluir).
    pub desfase: usize,
}

impl Default for Opciones {
    fn default() -> Self {
        Self {
            ancho: Ancho::Fijo(60),
            aspecto: 2.0,
            repetir: true,
            espacios: Espacios::Normal,
            umbral: None,
            invertir: false,
            huecos: false,
            suavizar: 0,
            desfase: 0,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Imagen(image::ImageError),
    TextoVacio,
    SiluetaVacia,
    OpcionInvalida(&'static str),
    /// El texto no cabe ni con `ANCHO_AUTO_MAX` columnas.
    TextoLargo {
        letras: usize,
        maximo: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Imagen(e) => write!(f, "no se pudo leer la imagen: {e}"),
            Error::TextoVacio => write!(f, "el texto está vacío"),
            Error::SiluetaVacia => write!(f, "no se encontró ninguna silueta en la imagen"),
            Error::OpcionInvalida(m) => write!(f, "opción inválida: {m}"),
            Error::TextoLargo { letras, maximo } => write!(
                f,
                "el texto tiene {letras} letras y la silueta admite como mucho {maximo} \
                 (con {ANCHO_AUTO_MAX} columnas)"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::Imagen(e)
    }
}

/// Máscara binaria: `true` = píxel de figura.
pub struct Mascara {
    pub ancho: usize,
    pub alto: usize,
    pub datos: Vec<bool>,
}

impl Mascara {
    fn get(&self, x: usize, y: usize) -> bool {
        self.datos[y * self.ancho + x]
    }

    /// La misma figura escalada `s` veces respecto al centro, en el mismo
    /// lienzo (vecino más cercano). Con `s < 1` nada se sale del lienzo.
    fn escalar(&self, s: f32) -> Mascara {
        let (cx, cy) = (self.ancho as f32 / 2.0, self.alto as f32 / 2.0);
        let datos = (0..self.alto)
            .flat_map(|y| (0..self.ancho).map(move |x| (x, y)))
            .map(|(x, y)| {
                let sx = cx + (x as f32 + 0.5 - cx) / s;
                let sy = cy + (y as f32 + 0.5 - cy) / s;
                sx >= 0.0
                    && sy >= 0.0
                    && (sx as usize) < self.ancho
                    && (sy as usize) < self.alto
                    && self.get(sx as usize, sy as usize)
            })
            .collect();
        Mascara {
            ancho: self.ancho,
            alto: self.alto,
            datos,
        }
    }
}

pub fn desde_bytes(bytes: &[u8], texto: &str, op: &Opciones) -> Result<String, Error> {
    dibujar(&image::load_from_memory(bytes)?, texto, op)
}

pub fn desde_ruta(ruta: &std::path::Path, texto: &str, op: &Opciones) -> Result<String, Error> {
    dibujar(&image::open(ruta)?, texto, op)
}

pub fn dibujar(img: &DynamicImage, texto: &str, op: &Opciones) -> Result<String, Error> {
    let silueta = Silueta::nueva(img, op)?;
    let texto = Texto::nuevo(texto, op.espacios, op.repetir)?;
    let columnas = silueta.columnas(op.ancho, texto.letras)?;
    let mut fotos = componer(&[silueta.rejilla(columnas)], &texto, &[op.desfase])?;
    Ok(fotos.remove(0))
}

/// El texto listo para escribirse: grafemas del ciclo (con el separador de
/// repetición si toca) y cuántos son del texto en sí.
pub(crate) struct Texto {
    grafemas: Vec<String>,
    repetir: bool,
    /// Letras del texto sin el separador (lo que cuenta `ancho="auto"`).
    pub(crate) letras: usize,
}

impl Texto {
    pub(crate) fn nuevo(texto: &str, espacios: Espacios, repetir: bool) -> Result<Self, Error> {
        if texto.trim().is_empty() {
            return Err(Error::TextoVacio);
        }
        let mut grafemas: Vec<String> = limpiar(texto, espacios)
            .graphemes(true)
            .map(String::from)
            .collect();
        let letras = grafemas.len();
        if repetir && espacios == Espacios::Normal {
            grafemas.push(" ".into());
        }
        Ok(Self {
            grafemas,
            repetir,
            letras,
        })
    }

    /// Largo de un ciclo completo: tras tantas letras el texto se repite igual.
    pub(crate) fn ciclo(&self) -> usize {
        self.grafemas.len()
    }

    fn en(&self, i: usize) -> &str {
        if self.repetir {
            &self.grafemas[i % self.grafemas.len()]
        } else {
            self.grafemas.get(i).map_or(" ", String::as_str)
        }
    }
}

/// Escribe el texto en cada rejilla (fotograma), empezando en su desfase.
/// Todas se recortan con los mismos márgenes (la unión de sus figuras), así
/// que los fotogramas quedan alineados entre sí.
pub(crate) fn componer(
    rejillas: &[Vec<Vec<bool>>],
    texto: &Texto,
    desfases: &[usize],
) -> Result<Vec<String>, Error> {
    let con_figura = |f: &Vec<bool>| f.contains(&true);
    let primera = rejillas
        .iter()
        .filter_map(|r| r.iter().position(con_figura))
        .min()
        .ok_or(Error::SiluetaVacia)?;
    let ultima = rejillas
        .iter()
        .filter_map(|r| r.iter().rposition(con_figura))
        .max()
        .unwrap();
    let margen = rejillas
        .iter()
        .flat_map(|r| r.iter().filter_map(|f| f.iter().position(|&b| b)))
        .min()
        .unwrap();

    Ok(rejillas
        .iter()
        .zip(desfases)
        .map(|(rejilla, &desfase)| {
            let mut i = desfase;
            (primera..=ultima)
                .map(|y| {
                    let fila = rejilla.get(y).map_or(&[][..], Vec::as_slice);
                    let linea: String = fila
                        .iter()
                        .skip(margen)
                        .map(|&b| {
                            if b {
                                i += 1;
                                texto.en(i - 1)
                            } else {
                                " "
                            }
                        })
                        .collect();
                    linea.trim_end().to_string()
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect())
}

/// Texto tal como se escribirá en las celdas (sin el separador de repetición).
fn limpiar(texto: &str, espacios: Espacios) -> String {
    match espacios {
        Espacios::Normal => texto.split_whitespace().collect::<Vec<_>>().join(" "),
        Espacios::Sin => texto.split_whitespace().collect(),
        // El salto final de un archivo no es un espacio que se haya "puesto".
        Espacios::Todos => texto
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect(),
    }
}

/// Celdas que ocupa el texto una vez (grafemas, espacios incluidos).
pub fn contar_letras(texto: &str, espacios: Espacios) -> usize {
    limpiar(texto, espacios).graphemes(true).count()
}

/// Máscara de una imagen lista para muestrearse a distintos anchos.
pub struct Silueta {
    mascara: Mascara,
    aspecto: f32,
}

impl Silueta {
    pub fn nueva(img: &DynamicImage, op: &Opciones) -> Result<Self, Error> {
        if op.ancho == Ancho::Fijo(0) {
            return Err(Error::OpcionInvalida("ancho debe ser > 0"));
        }
        if !(op.aspecto.is_finite() && op.aspecto > 0.0) {
            return Err(Error::OpcionInvalida("aspecto debe ser > 0"));
        }
        Ok(Self {
            mascara: mascara(&img.to_rgba8(), op),
            aspecto: op.aspecto,
        })
    }

    pub fn rejilla(&self, columnas: usize) -> Vec<Vec<bool>> {
        rejilla(&self.mascara, columnas, self.aspecto)
    }

    /// Rejilla de la figura escalada `escala` veces (1.0 = tamaño original).
    pub fn rejilla_escalada(&self, columnas: usize, escala: f32) -> Vec<Vec<bool>> {
        if escala == 1.0 {
            return self.rejilla(columnas);
        }
        rejilla(&self.mascara.escalar(escala), columnas, self.aspecto)
    }

    /// Resuelve `Ancho::Auto` para un texto de `letras` letras.
    pub fn columnas(&self, ancho: Ancho, letras: usize) -> Result<usize, Error> {
        match ancho {
            Ancho::Fijo(n) => Ok(n),
            Ancho::Auto => self.ancho_para(letras),
        }
    }

    /// Letras que caben a `columnas` de ancho.
    pub fn capacidad(&self, columnas: usize) -> usize {
        self.rejilla(columnas)
            .iter()
            .flatten()
            .filter(|&&b| b)
            .count()
    }

    /// Menor ancho con capacidad >= `letras` (búsqueda binaria; la capacidad
    /// crece con el ancho salvo pequeñas oscilaciones de muestreo).
    pub fn ancho_para(&self, letras: usize) -> Result<usize, Error> {
        if letras == 0 {
            return Err(Error::TextoVacio);
        }
        let maximo = self.capacidad(ANCHO_AUTO_MAX);
        if maximo == 0 {
            return Err(Error::SiluetaVacia);
        }
        if maximo < letras {
            return Err(Error::TextoLargo { letras, maximo });
        }
        let (mut lo, mut hi) = (1, ANCHO_AUTO_MAX);
        while lo < hi {
            let medio = (lo + hi) / 2;
            if self.capacidad(medio) >= letras {
                hi = medio;
            } else {
                lo = medio + 1;
            }
        }
        Ok(lo)
    }

    pub fn analizar(&self, columnas: usize) -> Result<Analisis, Error> {
        let rejilla = self.rejilla(columnas);
        let (filas, margen) = recortar(&rejilla)?;
        let por_fila: Vec<usize> = filas
            .iter()
            .map(|f| f.iter().filter(|&&b| b).count())
            .collect();
        let tramos: Vec<usize> = filas
            .iter()
            .flat_map(|f| f.split(|&b| !b).map(<[bool]>::len).filter(|&n| n > 0))
            .collect();
        let plantilla = filas
            .iter()
            .map(|f| {
                let l: String = f[margen..]
                    .iter()
                    .map(|&b| if b { '#' } else { ' ' })
                    .collect();
                l.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Analisis {
            ancho: columnas,
            filas: filas.len(),
            letras: por_fila.iter().sum(),
            por_fila,
            tramos: tramos.len(),
            tramo_min: tramos.iter().copied().min().unwrap_or(0),
            tramo_max: tramos.iter().copied().max().unwrap_or(0),
            plantilla,
        })
    }
}

/// Resumen de la silueta a un ancho dado, para diseñar el texto de antemano.
#[derive(Debug, Clone)]
pub struct Analisis {
    pub ancho: usize,
    pub filas: usize,
    /// Capacidad total en letras (los espacios también ocupan una celda).
    pub letras: usize,
    pub por_fila: Vec<usize>,
    /// Tramos continuos de celdas: los huecos entre tramos parten palabras.
    pub tramos: usize,
    pub tramo_min: usize,
    pub tramo_max: usize,
    /// Forma con `#` en cada celda que recibirá una letra.
    pub plantilla: String,
}

impl Analisis {
    /// Palabras aproximadas, suponiendo ~5 letras + 1 espacio por palabra.
    pub fn palabras_aprox(&self) -> usize {
        self.letras / 6
    }
}

/// Filas entre la primera y la última con figura, y el margen izquierdo común.
fn recortar(rejilla: &[Vec<bool>]) -> Result<(&[Vec<bool>], usize), Error> {
    let primera = rejilla
        .iter()
        .position(|f| f.contains(&true))
        .ok_or(Error::SiluetaVacia)?;
    let ultima = rejilla.iter().rposition(|f| f.contains(&true)).unwrap();
    let filas = &rejilla[primera..=ultima];
    let margen = filas
        .iter()
        .filter_map(|f| f.iter().position(|&b| b))
        .min()
        .unwrap();
    Ok((filas, margen))
}

/// Separa figura de fondo. Con transparencia usa el alfa; si no, compara cada
/// píxel con el color de fondo estimado en los bordes. La luminancia sola no
/// sirve: un amarillo claro sobre blanco se perdería.
pub fn mascara(img: &RgbaImage, op: &Opciones) -> Mascara {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let tiene_alfa = img.pixels().any(|p| p[3] < 250);

    let mut datos: Vec<bool> = if tiene_alfa {
        img.pixels().map(|p| p[3] >= 128).collect()
    } else {
        let fondo = color_fondo(img);
        let dist: Vec<u8> = img
            .pixels()
            .map(|p| {
                let d: f32 = (0..3)
                    .map(|i| (p[i] as f32 - fondo[i] as f32).powi(2))
                    .sum::<f32>()
                    .sqrt();
                // Distancia máxima RGB ≈ 441.7 -> escalar a 0..=255.
                (d * 255.0 / 441.7).round().min(255.0) as u8
            })
            .collect();
        let t = op.umbral.unwrap_or_else(|| umbral_auto(&dist, w, h));
        dist.iter().map(|&d| d > t).collect()
    };

    // Cerrar antes de rellenar huecos: así un contorno punteado queda sellado.
    if op.suavizar > 0 {
        cerrar(&mut datos, w, h, op.suavizar);
    }
    if !op.huecos {
        rellenar_huecos(&mut datos, w, h);
    }
    if op.invertir {
        datos.iter_mut().for_each(|b| *b = !*b);
    }
    Mascara {
        ancho: w,
        alto: h,
        datos,
    }
}

/// Cierre morfológico con un cuadrado de lado `2r+1`: dilatar y luego
/// erosionar, cada uno separable en una pasada horizontal y otra vertical.
fn cerrar(datos: &mut [bool], w: usize, h: usize, r: usize) {
    for erosion in [false, true] {
        pasada(datos, w, h, r, 1, w, erosion);
        pasada(datos, h, w, r, w, 1, erosion);
    }
}

/// Aplica una ventana de radio `r` a lo largo de líneas de `largo` elementos
/// separados por `paso`; `lineas` líneas que empiezan cada `salto`. Dilatar =
/// algún vecino es figura; erosionar = todos lo son (ignorando fuera de rango).
fn pasada(
    datos: &mut [bool],
    largo: usize,
    lineas: usize,
    r: usize,
    paso: usize,
    salto: usize,
    erosion: bool,
) {
    let mut acum = vec![0usize; largo + 1];
    for l in 0..lineas {
        let base = l * salto;
        for i in 0..largo {
            acum[i + 1] = acum[i] + datos[base + i * paso] as usize;
        }
        for i in 0..largo {
            let (a, b) = (i.saturating_sub(r), (i + r + 1).min(largo));
            let n = acum[b] - acum[a];
            datos[base + i * paso] = if erosion { n == b - a } else { n > 0 };
        }
    }
}

/// Marca como figura todo el fondo que no esté conectado (4-vecinos) con el
/// borde de la imagen.
fn rellenar_huecos(datos: &mut [bool], w: usize, h: usize) {
    let mut exterior = vec![false; datos.len()];
    let mut pila: Vec<usize> = (0..w)
        .flat_map(|x| [x, (h - 1) * w + x])
        .chain((0..h).flat_map(|y| [y * w, y * w + w - 1]))
        .collect();
    while let Some(i) = pila.pop() {
        if datos[i] || exterior[i] {
            continue;
        }
        exterior[i] = true;
        let (x, y) = (i % w, i / w);
        if x > 0 {
            pila.push(i - 1);
        }
        if x + 1 < w {
            pila.push(i + 1);
        }
        if y > 0 {
            pila.push(i - w);
        }
        if y + 1 < h {
            pila.push(i + w);
        }
    }
    for (d, e) in datos.iter_mut().zip(exterior) {
        *d = !e;
    }
}

/// Mediana por canal de los píxeles del borde.
fn color_fondo(img: &RgbaImage) -> [u8; 3] {
    let (w, h) = (img.width(), img.height());
    let mut canales: [Vec<u8>; 3] = Default::default();
    let mut push = |x, y| {
        let p = img.get_pixel(x, y);
        for i in 0..3 {
            canales[i].push(p[i]);
        }
    };
    for x in 0..w {
        push(x, 0);
        push(x, h - 1);
    }
    for y in 0..h {
        push(0, y);
        push(w - 1, y);
    }
    canales.map(|mut c| {
        c.sort_unstable();
        c[c.len() / 2]
    })
}

/// Otsu tiende a cortar entre los tonos de la figura (p. ej. gris oscuro vs.
/// sombra clara), dejando los bordes suaves como fondo y abriendo fugas al
/// rellenar huecos. Se limita al ruido del fondo medido en el borde + margen.
fn umbral_auto(dist: &[u8], w: usize, h: usize) -> u8 {
    let mut borde: Vec<u8> = (0..w)
        .flat_map(|x| [dist[x], dist[(h - 1) * w + x]])
        .chain((0..h).flat_map(|y| [dist[y * w], dist[y * w + w - 1]]))
        .collect();
    borde.sort_unstable();
    let ruido = borde[borde.len() * 99 / 100];
    otsu(dist).min(ruido.saturating_add(10))
}

/// Umbral de Otsu sobre valores 0-255.
fn otsu(valores: &[u8]) -> u8 {
    let mut hist = [0u64; 256];
    for &v in valores {
        hist[v as usize] += 1;
    }
    let total = valores.len() as f64;
    let suma: f64 = hist
        .iter()
        .enumerate()
        .map(|(i, &c)| i as f64 * c as f64)
        .sum();
    let (mut peso_b, mut suma_b, mut mejor, mut t) = (0.0, 0.0, -1.0, 0u8);
    for (i, &c) in hist.iter().enumerate() {
        peso_b += c as f64;
        if peso_b == 0.0 {
            continue;
        }
        let peso_f = total - peso_b;
        if peso_f == 0.0 {
            break;
        }
        suma_b += i as f64 * c as f64;
        let media_b = suma_b / peso_b;
        let media_f = (suma - suma_b) / peso_f;
        let var = peso_b * peso_f * (media_b - media_f).powi(2);
        if var > mejor {
            mejor = var;
            t = i as u8;
        }
    }
    t
}

/// Reduce la máscara a `columnas` celdas; una celda es figura si al menos la
/// mitad de sus píxeles lo son.
pub fn rejilla(m: &Mascara, columnas: usize, aspecto: f32) -> Vec<Vec<bool>> {
    let celda_w = m.ancho as f32 / columnas as f32;
    let celda_h = celda_w * aspecto;
    let filas = ((m.alto as f32 / celda_h).round() as usize).max(1);

    let rango = |i: usize, paso: f32, max: usize| {
        let a = ((i as f32 * paso) as usize).min(max - 1);
        let b = (((i + 1) as f32 * paso) as usize).clamp(a + 1, max);
        a..b
    };

    (0..filas)
        .map(|f| {
            let ys = rango(f, celda_h, m.alto);
            (0..columnas)
                .map(|c| {
                    let xs = rango(c, celda_w, m.ancho);
                    let total = xs.len() * ys.len();
                    let llenos = ys
                        .clone()
                        .flat_map(|y| xs.clone().map(move |x| (x, y)))
                        .filter(|&(x, y)| m.get(x, y))
                        .count();
                    llenos * 2 >= total
                })
                .collect()
        })
        .collect()
}

/// Escribe el texto (por grafemas, espacios según `espacios`) en las celdas
/// de figura, fila a fila. Recorta filas vacías y el margen izquierdo común.
pub fn rellenar(
    rejilla: &[Vec<bool>],
    texto: &str,
    repetir: bool,
    espacios: Espacios,
) -> Result<String, Error> {
    let texto = Texto::nuevo(texto, espacios, repetir)?;
    let mut fotos = componer(&[rejilla.to_vec()], &texto, &[0])?;
    Ok(fotos.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn circulo(lado: u32, fondo: Rgba<u8>, tinta: Rgba<u8>) -> DynamicImage {
        let c = lado as f32 / 2.0;
        let r = lado as f32 * 0.4;
        DynamicImage::ImageRgba8(RgbaImage::from_fn(lado, lado, |x, y| {
            let d = ((x as f32 - c).powi(2) + (y as f32 - c).powi(2)).sqrt();
            if d < r {
                tinta
            } else {
                fondo
            }
        }))
    }

    const BLANCO: Rgba<u8> = Rgba([255, 255, 255, 255]);
    const AMARILLO: Rgba<u8> = Rgba([255, 212, 59, 255]);

    #[test]
    fn circulo_claro_sobre_blanco_se_detecta() {
        let op = Opciones {
            ancho: Ancho::Fijo(20),
            ..Default::default()
        };
        let s = dibujar(&circulo(200, BLANCO, AMARILLO), "ab", &op).unwrap();
        let lineas: Vec<&str> = s.lines().collect();
        // Silueta redonda: la fila central es más ancha que la primera.
        let medio = lineas[lineas.len() / 2].trim().chars().count();
        assert!(medio > lineas[0].trim().chars().count());
        assert!(medio >= 14, "fila central demasiado estrecha: {medio}");
        // aspecto 2.0: ~la mitad de filas que columnas.
        assert!((7..=9).contains(&lineas.len()), "filas: {}", lineas.len());
    }

    #[test]
    fn transparencia_manda_sobre_color() {
        let transp = Rgba([255, 255, 255, 0]);
        let tinta = Rgba([255, 255, 255, 255]); // mismo color que el fondo
        let op = Opciones {
            ancho: Ancho::Fijo(20),
            ..Default::default()
        };
        assert!(dibujar(&circulo(100, transp, tinta), "x", &op)
            .unwrap()
            .contains('x'));
    }

    #[test]
    fn texto_en_orden_y_con_tildes() {
        let rejilla = vec![vec![true; 4], vec![false, true, true, false]];
        assert_eq!(
            rellenar(&rejilla, "ñandú", false, Espacios::Normal).unwrap(),
            "ñand\n ú"
        );
    }

    #[test]
    fn repetir_cicla_con_espacio() {
        let rejilla = vec![vec![true; 7]];
        assert_eq!(
            rellenar(&rejilla, "abc", true, Espacios::Normal).unwrap(),
            "abc abc"
        );
        assert_eq!(
            rellenar(&rejilla, "abc", false, Espacios::Normal).unwrap(),
            "abc"
        );
    }

    #[test]
    fn colapsa_espacios_y_saltos() {
        let rejilla = vec![vec![true; 5]];
        assert_eq!(
            rellenar(&rejilla, "a\n\n  b", false, Espacios::Normal).unwrap(),
            "a b"
        );
    }

    #[test]
    fn espacios_sin_quita_todos_y_pega_repeticiones() {
        let rejilla = vec![vec![true; 10]];
        let r = rellenar(&rejilla, "Git  hub\n", true, Espacios::Sin).unwrap();
        assert_eq!(r, "GithubGith");
    }

    #[test]
    fn espacios_todos_conserva_cada_espacio() {
        let rejilla = vec![vec![true; 12]];
        let r = rellenar(&rejilla, "a  b\nc\n", true, Espacios::Todos).unwrap();
        // "a␣␣b␣c" y se repite sin separador añadido; el \n final no cuenta.
        assert_eq!(r, "a  b ca  b c");
        assert_eq!(contar_letras("a  b\nc\r\n", Espacios::Todos), 6);
        assert!(matches!(
            rellenar(&rejilla, " \n ", true, Espacios::Todos),
            Err(Error::TextoVacio)
        ));
    }

    #[test]
    fn recorta_margenes() {
        let rejilla = vec![vec![false; 3], vec![false, true, false], vec![false; 3]];
        assert_eq!(
            rellenar(&rejilla, "z", false, Espacios::Normal).unwrap(),
            "z"
        );
    }

    #[test]
    fn errores() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(10, 10, BLANCO));
        assert!(matches!(
            dibujar(&img, "hola", &Opciones::default()),
            Err(Error::SiluetaVacia)
        ));
        let img = circulo(50, BLANCO, AMARILLO);
        assert!(matches!(
            dibujar(&img, "  \n", &Opciones::default()),
            Err(Error::TextoVacio)
        ));
        let op = Opciones {
            ancho: Ancho::Fijo(0),
            ..Default::default()
        };
        assert!(matches!(
            dibujar(&img, "a", &op),
            Err(Error::OpcionInvalida(_))
        ));
    }

    /// Anillo oscuro con interior del mismo color que el fondo.
    fn anillo(lado: u32) -> DynamicImage {
        let c = lado as f32 / 2.0;
        DynamicImage::ImageRgba8(RgbaImage::from_fn(lado, lado, |x, y| {
            let d = ((x as f32 - c).powi(2) + (y as f32 - c).powi(2)).sqrt();
            if (lado as f32 * 0.3..lado as f32 * 0.4).contains(&d) {
                Rgba([40, 40, 40, 255])
            } else {
                BLANCO
            }
        }))
    }

    #[test]
    fn interior_del_color_del_fondo_es_figura() {
        let op = Opciones {
            ancho: Ancho::Fijo(20),
            ..Default::default()
        };
        let s = dibujar(&anillo(200), "o", &op).unwrap();
        let lineas: Vec<&str> = s.lines().collect();
        // Solo los espacios simples del separador de repetición, sin hueco.
        assert!(!lineas[lineas.len() / 2].trim().contains("  "), "{s}");
    }

    #[test]
    fn huecos_true_respeta_el_interior() {
        let op = Opciones {
            ancho: Ancho::Fijo(20),
            huecos: true,
            ..Default::default()
        };
        let s = dibujar(&anillo(200), "o", &op).unwrap();
        let lineas: Vec<&str> = s.lines().collect();
        assert!(lineas[lineas.len() / 2].trim().contains("  "), "{s}");
    }

    #[test]
    fn invertir_escribe_en_el_fondo() {
        let op = Opciones {
            ancho: Ancho::Fijo(20),
            invertir: true,
            ..Default::default()
        };
        let s = dibujar(&circulo(200, BLANCO, AMARILLO), "o", &op).unwrap();
        let lineas: Vec<&str> = s.lines().collect();
        let medio = lineas[lineas.len() / 2];
        // En el centro hay un hueco rodeado de texto.
        assert!(medio.starts_with('o') && medio.trim_end().ends_with('o') && medio.contains("  "));
    }

    #[test]
    fn suavizar_une_trazos_punteados() {
        // Barra de puntos de 2px separados 2px, de x=12 a x=89.
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(100, 40, |x, y| {
            if (10..30).contains(&y) && (10..90).contains(&x) && x % 4 < 2 {
                Rgba([0, 0, 0, 255])
            } else {
                BLANCO
            }
        }));
        let img = img.to_rgba8();
        let punteada = mascara(&img, &Opciones::default());
        let op = Opciones {
            suavizar: 2,
            ..Default::default()
        };
        let cerrada = mascara(&img, &op);
        let fila = |m: &Mascara| (12..90).filter(|&x| m.get(x, 20)).count();
        assert_eq!(fila(&punteada), 40);
        assert_eq!(fila(&cerrada), 78);
        // El cierre no engorda la figura: fuera de la barra sigue siendo fondo.
        assert!(!cerrada.get(5, 20) && !cerrada.get(50, 5));
    }

    #[test]
    fn ancho_auto_cabe_el_texto_una_vez() {
        let img = circulo(200, BLANCO, AMARILLO);
        let texto = "Podrá nublarse el sol eternamente; podrá secarse en un instante el mar";
        let op = Opciones {
            ancho: Ancho::Auto,
            repetir: false,
            ..Default::default()
        };
        let s = dibujar(&img, texto, &op).unwrap();
        // Todo el texto aparece, en orden, sin cortarse.
        let sin_espacios: String = s.split_whitespace().collect();
        assert_eq!(sin_espacios, texto.split_whitespace().collect::<String>());
        // Y es el menor ancho posible: uno menos ya no alcanza.
        let sil = Silueta::nueva(&img, &op).unwrap();
        let n = sil
            .ancho_para(contar_letras(texto, Espacios::Normal))
            .unwrap();
        assert!(sil.capacidad(n) >= contar_letras(texto, Espacios::Normal));
        assert!(sil.capacidad(n - 1) < contar_letras(texto, Espacios::Normal));
    }

    #[test]
    fn ancho_auto_texto_demasiado_largo() {
        let img = circulo(20, BLANCO, AMARILLO);
        let sil = Silueta::nueva(&img, &Opciones::default()).unwrap();
        let err = sil.ancho_para(usize::MAX).unwrap_err();
        assert!(matches!(err, Error::TextoLargo { .. }));
    }

    #[test]
    fn analisis_cuadra_con_el_dibujo() {
        let img = anillo(200);
        let op = Opciones {
            huecos: true,
            ..Default::default()
        };
        let a = Silueta::nueva(&img, &op).unwrap().analizar(30).unwrap();
        assert_eq!(a.letras, a.por_fila.iter().sum::<usize>());
        assert_eq!(a.filas, a.plantilla.lines().count());
        assert_eq!(a.letras, a.plantilla.matches('#').count());
        // El anillo con hueco tiene filas con dos tramos.
        assert!(a.tramos > a.filas);
        // La plantilla tiene la misma forma que el dibujo.
        let op = Opciones {
            ancho: Ancho::Fijo(30),
            huecos: true,
            repetir: false,
            ..Default::default()
        };
        let s = dibujar(&img, &"x".repeat(a.letras), &op).unwrap();
        assert_eq!(s.replace('x', "#"), a.plantilla);
    }
}
