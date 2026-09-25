# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Estado

MVP funcional. Paquete/crate/módulo: **`caligrama`** (un caligrama es un poema cuyo texto dibuja una figura). API Python y CLI en español.

```python
import caligrama
print(caligrama.dibujar("flor.png", "Podrá nublarse el sol...", ancho=60))
# imagen: str | os.PathLike | bytes. ancho: int | "auto". Opcionales: aspecto=2.0, repetir=True, espacios="normal"|"sin"|"todos", umbral=None (auto), invertir=False, huecos=False, suavizar=0, color=False
caligrama.analizar("flor.png", texto=None, ancho=60)   # dict: letras, palabras_aprox, letras_por_fila, tramos, plantilla, letras_por_ancho, [ancho_recomendado, sobran]
```
```bash
caligrama flor.png -w 60 -t "mensaje"      # o -f poema.txt, o por stdin; -w auto
caligrama analizar flor.png [-f poema.txt] # informe para diseñar el texto (no lee stdin)
```

## Objetivo

Librería de Python escrita **100% en Rust** (PyO3 + maturin) que, de forma local y offline:

1. Recibe una imagen (ruta o bytes).
2. Detecta la silueta/contorno del objeto.
3. Rellena el interior de la silueta con un mensaje o poema, carácter a carácter, dejando espacios fuera.
4. Devuelve **solo un `str`** con saltos de línea cuya forma reproduce la imagen.

Debe poder usarse desde cualquier script Python (`import ...`) y desde la terminal.

Referencias de la idea en `../flor-amarilla/`:
- `archivo.txt`: arte ASCII por densidad generado con `pywhatkit.image_to_ascii_art` (80 columnas). Es la inspiración de formato, **no** el resultado buscado: aquí los caracteres son el texto del poema, no una rampa de densidad (`.:!*%$@&`).
- `test.jpeg`: imagen de prueba (logo de Python, 189x148, JPEG sin alpha, fondo blanco, dos colores azul/amarillo).
- `art.py`, `poema.py`: prototipos en Python (pywhatkit, Pillow). No se deben usar como dependencias.

## Restricciones

- Toda la lógica (decodificar imagen, segmentar, redimensionar, componer texto) vive en Rust. Nada de Pillow/numpy/pywhatkit en runtime; el paquete Python no debe tener dependencias.
- Decodificación de imágenes con crates de Rust (p. ej. `image`).
- El CLI también debe ser Rust: expuesto como `[project.scripts]` en `pyproject.toml` apuntando a un `#[pyfunction]` que lee `sys.argv`, o como binario Rust aparte. No escribir el CLI en Python.
- Usar `abi3` en PyO3 (feature `abi3-py39`: pyo3 0.29) para que una sola wheel funcione en cualquier versión de Python ≥ mínima.

## Arquitectura

- `src/core.rs`: núcleo puro Rust, sin PyO3. Pipeline `dibujar` = `mascara` → `rejilla` → `rellenar`. Tests unitarios aquí con imágenes sintéticas.
  - `mascara`: si la imagen tiene transparencia usa el alfa; si no, estima el fondo como la mediana de los píxeles del borde y umbraliza la distancia RGB con Otsu. La luminancia sola no sirve (en `tests/data/python.jpeg` el amarillo es casi tan claro como el fondo blanco).
    - Umbral automático = `min(Otsu, p99 del ruido en el borde + 10)`. Otsu solo corta entre tonos de la figura y deja bordes suaves como fondo.
    - `suavizar=N`: cierre morfológico (dilatar + erosionar, cuadrado de radio N px, separable con sumas prefijas) para unir trazos punteados o hechos de letras. Va **antes** del relleno de huecos: un contorno cerrado hace que su interior pase a ser figura; combinar con `huecos=True` para conservar el hueco.
    - Después rellena huecos: fondo = lo parecido al color del borde **y conectado al borde** (flood fill). Así una barriga blanca sobre fondo blanco sigue siendo figura. `huecos=True` / `--con-huecos` lo desactiva (p. ej. para conservar el ojo del logo de Python).
  - `rejilla`: celda = figura si ≥50 % de sus píxeles lo son; `aspecto` corrige que los caracteres de terminal son ~2x más altos que anchos.
  - `rellenar`: espacios según `Espacios` (`Normal` colapsa y separa repeticiones con un espacio; `Sin` los quita y pega repeticiones; `Todos` conserva cada uno, `\n`/tab = 1 espacio, sin separador, ignorando saltos finales). `contar_letras` usa el mismo modo, así que `ancho="auto"` y `analizar` cuentan igual que el dibujo. Además recorre por grafemas (`unicode-segmentation`), recorta filas vacías y el margen izquierdo común.
  - Color (`color=True` / `--colores imagen`): `Mascara` guarda también el RGB de cada píxel; `colores` promedia en luz lineal los píxeles de figura de cada celda (misma geometría que `rejilla` vía `muestrear`) y `componer` emite ANSI truecolor solo al cambiar de color. El color viaja dentro del `str`: `animacion::celdas` lo parsea para `a_svg` (`fill` por tramo) y `colorear` (un degradado explícito lo reemplaza).
  - `Silueta`: máscara calculada una vez y muestreada a varios anchos. `ancho_para` = búsqueda binaria del menor ancho cuya capacidad (celdas de figura) ≥ letras del texto, hasta `ANCHO_AUTO_MAX`. `analizar` y `dibujar` usan la misma rejilla y recorte, así que `plantilla` coincide celda a celda con el dibujo (hay test de eso).
- `src/animacion.rs`: `animar` (fotogramas alineados: `paso` = letras que avanza el texto vía `desfase`, `latido` = escala de la máscara con pulso doble; sin `fotogramas`, ciclo/mcd(ciclo, paso) para bucle sin costura), `a_svg` (cada palabra con `x` explícita, frames con CSS `step-end` + `opacity="0"` para visores sin CSS) y `colorear` (ANSI truecolor). Usa `core::Texto` y `core::componer`, que recorta todos los fotogramas con los mismos márgenes (unión de figuras): por eso quedan alineados.
- `src/lib.rs`: binding fino. Convierte `core::Error` a `OSError` (E/S) o `ValueError`. `informe` alimenta tanto `analizar` (dict) como `caligrama analizar` (texto). `reproducir_en` es el bucle de la terminal: llama a `py.check_signals()` en cada fotograma para que Ctrl+C funcione dentro de Rust y siempre restaura el cursor. Contiene también el CLI (`cli` lee `sys.argv`, devuelve el código de salida); `pyproject.toml` lo expone como `[project.scripts] caligrama = "caligrama:cli"`. El parseo de argumentos es manual para no añadir dependencias.
- En el enum `Imagen`, `Bytes` debe ir antes que `Ruta`: `PathBuf` también acepta `bytes` al extraer.
- `tests/test_caligrama.py`: tests de la API Python y del CLI (vía subprocess). `tests/data/referencia_ascii.txt` es la salida de pywhatkit de referencia (80×34).

## Comandos

El entorno tiene conda activo (`CONDA_PREFIX`); maturin se niega a correr si también está `VIRTUAL_ENV`, y sin venv activo instala en el env de conda. Para trabajar en `.venv`:

```bash
uv venv .venv && uv pip install --python .venv/bin/python maturin pytest   # una vez
unset CONDA_PREFIX; export VIRTUAL_ENV=$PWD/.venv PATH=$PWD/.venv/bin:$PATH

maturin develop --uv       # compila e instala en .venv (--release para medir rendimiento)
maturin build --release    # wheel abi3 (cp39-abi3) en target/wheels/ (o en $CARGO_TARGET_DIR)

cargo test                 # tests del núcleo Rust
cargo test rellenar        # filtra tests Rust por nombre
pytest -q tests            # API Python + CLI (requiere maturin develop antes)
pytest tests/test_caligrama.py::test_errores

cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Documentación web

La documentación completa (ES/EN) vive en otro repo: `../maosuarez-profile/public/docs/caligrama.html`, publicada en https://www.maosuarez.com/docs/caligrama. Es un solo HTML con ambos idiomas (`<article data-l="es">` y `<article data-l="en">`, más el índice `nav.toc`).

- **Después de cada cambio** que afecte a la API, el CLI, los valores por defecto, los mensajes de error o la versión, actualiza esa página en **los dos idiomas** (incluida la versión del encabezado `.ver` y `caligrama.__version__` en "Tipos y versión"). Los ejemplos de salida se sacan ejecutando caligrama, no se inventan.
- Si no es posible editarla (el repo no está disponible o no hay permisos), **avisa explícitamente** al terminar de que la documentación de `maosuarez-profile` debe actualizarse y con qué.
- Los cambios en `maosuarez-profile` se commitean y empujan en ese repo, no en este.

## Publicación

- Repo: https://github.com/maosuarez/caligrama · PyPI: https://pypi.org/p/caligrama
- `.github/workflows/ci.yml`: fmt, clippy, `cargo test` y pytest + `examples/demo.py` en Linux/macOS/Windows × Python 3.9/3.13.
- `.github/workflows/release.yml`: al empujar un tag `vX.Y.Z` compila wheels abi3 (manylinux/musllinux x86_64+aarch64, Windows x64, macOS x86_64+arm64) y sdist, publica en PyPI con Trusted Publishing (environment `pypi`, sin tokens) y crea el GitHub Release. La versión sale de `Cargo.toml`.
- `caligrama.pyi` son las firmas públicas: actualizarlas junto con `src/lib.rs`.
- `assets/latido.svg` y `assets/titulo.svg` (animaciones del README) se regeneran con `python examples/latido.py --svg assets/latido.svg` y `python examples/titulo.py --svg assets/titulo.svg`; `assets/arbol.svg` (árbol a color) con `python examples/arbol.py --svg assets/arbol.svg`.
- `bench/benchmark.py` compara contra pywhatkit, ascii_magic, viu, catimg y scripts a mano con Pillow (`pip install -r bench/requirements.txt`; los que falten se omiten). Escribe `bench/resultados/` (ignorado por git). Las cifras de la tabla del README salen de ahí. `assets/titulo.png` ya viene limpio (sin las líneas finas de la sombra de la tipografía original). El README usa URLs absolutas de GitHub porque también se muestra en PyPI.
- `live-test/` está en `.gitignore`: carpeta local para probar imágenes (`./live-test/probar.sh [ancho] [opciones]`).
