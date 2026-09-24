<div align="center">

# caligrama

**Dale una imagen y un poema. Te devuelve el poema con la forma de la imagen.**

[![PyPI](https://img.shields.io/pypi/v/caligrama?color=ff2d55)](https://pypi.org/project/caligrama/)
[![Python](https://img.shields.io/pypi/pyversions/caligrama)](https://pypi.org/project/caligrama/)
[![CI](https://github.com/maosuarez/caligrama/actions/workflows/ci.yml/badge.svg)](https://github.com/maosuarez/caligrama/actions/workflows/ci.yml)
[![Licencia MIT](https://img.shields.io/badge/licencia-MIT-blue)](https://github.com/maosuarez/caligrama/blob/main/LICENSE)
[![Hecho en Rust](https://img.shields.io/badge/núcleo-Rust-orange)](https://github.com/maosuarez/caligrama/tree/main/src)

<img src="https://raw.githubusercontent.com/maosuarez/caligrama/main/assets/latido.svg" alt="Un corazón que late, escrito con un poema que corre por dentro. Generado con caligrama." width="420">

<sub>Cada uno de los 40 fotogramas es una llamada a <code>caligrama.dibujar()</code>. El código está en <a href="https://github.com/maosuarez/caligrama/blob/main/examples/latido.py"><code>examples/latido.py</code></a>.</sub>

</div>

Un caligrama es un poema cuyas letras dibujan aquello de lo que habla. Apollinaire los hacía a mano. Esta librería los hace con cualquier imagen que le pases: detecta la silueta, la reduce a una rejilla de caracteres y escribe tu texto dentro, letra por letra y en orden. Lo que sale es texto plano, así que puedes pegarlo en un chat, en un README, en un commit o en una tarjeta.

```text
$ caligrama logo.jpeg -w auto --sin-repetir -f becquer.txt

         Podrá nubla
        rse el sol et
       ernamente; podr
        á secarse en u
  n instante el mar; podrá r
 omperse el eje de la  tierr
a como un débil crist al. ¡To
do sucede rá! Podrá la muerte
 cubrir me con su fúnebre cre
 spón;  pero jamás en mí pod
  rá ap agarse la llama de
        tu amor.
```

## Instalación

```bash
pip install caligrama
```

Eso es todo. No arrastra dependencias de Python (ni Pillow, ni numpy): la lectura de la imagen y el dibujo se hacen en Rust, dentro de una sola wheel que sirve para Python 3.9 en adelante en Linux, macOS y Windows. Funciona sin conexión y no manda nada a ningún lado.

## En diez segundos

Desde Python:

```python
import caligrama

print(caligrama.dibujar("gato.png", "Te quiero más que a mi café de la mañana", ancho=50))
```

Desde la terminal:

```bash
caligrama gato.png -w 50 -t "Te quiero más que a mi café de la mañana"
caligrama gato.png -f poema.txt          # el texto desde un archivo
cat poema.txt | caligrama gato.png       # o desde stdin
```

La imagen puede ser una ruta, un `pathlib.Path` o los bytes crudos (lo que te devuelve `requests`, una base de datos o un upload). Lee PNG, JPEG, GIF, BMP y WebP.

## Diseña el texto antes de escribirlo

Lo más frustrante de hacer caligramas a mano es escribir el poema y descubrir que no cabe, o que sobra media figura. `caligrama analizar` te dice de antemano cuánto espacio tienes:

```text
$ caligrama analizar logo.jpeg -w 40 -t "Podrá nublarse el sol eternamente"

Silueta a 40 columnas × 16 filas
Caben 452 letras (los espacios cuentan) ≈ 75 palabras
Tramos: 26 (de 5 a 28 letras); cada hueco entre tramos puede partir una palabra

Plantilla (letras por fila a la derecha):
               #############               │  13
             #################             │  17
            ###################            │  19
     ########################## ######     │  32
    ########################### #######    │  34
  ...

Otros anchos:
  ancho  filas  letras  ≈palabras
     30     12     252        42
     40     16     452        75
     60     23     955       159
  ...

Tu texto: 33 letras. Cabe completo desde 11 columnas (-w auto). A 40 columnas sobran 419 letras.
```

La plantilla coincide celda por celda con el dibujo final (hay un test que lo garantiza), así que puedes contar letras por fila y ajustar tus versos a mano. Desde Python, `caligrama.analizar()` devuelve lo mismo en un diccionario.

Y si no quieres contar nada, `ancho="auto"` busca el ancho exacto en el que tu texto llena la figura una sola vez.

## Por qué se ve bien con imágenes reales

Casi todas las herramientas de arte ASCII deciden qué es figura mirando el brillo: lo oscuro se pinta y lo claro se deja vacío. Eso falla más de lo que parece. En el logo de Python, la serpiente amarilla es casi tan clara como el fondo blanco y desaparece. En un pingüino, la barriga blanca queda como un agujero.

caligrama hace otra cosa:

1. Estima el color del fondo con los píxeles del borde de la imagen y marca como figura lo que se aleja de ese color, con un umbral automático que se ajusta al ruido del JPEG.
2. Rellena lo que está encerrado por la figura aunque sea del color del fondo. La barriga del pingüino vuelve a ser pingüino. Si quieres conservar los huecos (el ojo del logo, el centro de una dona), usa `huecos=True`.
3. Si la imagen tiene transparencia, usa el canal alfa y listo.
4. `suavizar=N` une trazos punteados o hechos de letras antes de muestrear, útil para logos de línea fina o imágenes que ya son arte ASCII.
5. Corrige la proporción de la terminal (un carácter es el doble de alto que de ancho) para que un círculo salga redondo.
6. Recorre el texto por grafemas, no por bytes: tildes, ñ, ü y emojis compuestos no se parten.

## Todas las opciones

| Python | Terminal | Por defecto | Qué hace |
|---|---|---|---|
| `ancho` | `-w, --ancho` | `60` | Columnas de salida. `"auto"` elige el menor ancho donde cabe todo el texto. |
| `repetir` | `--sin-repetir` | `True` | Repite el texto hasta llenar la figura. |
| `espacios` | `--espacios` | `"normal"` | `normal` junta espacios y separa repeticiones con uno; `sin` los quita todos (figura más sólida); `todos` los deja tal cual. |
| `huecos` | `--con-huecos` | `False` | Respeta los huecos interiores del color del fondo. |
| `suavizar` | `--suavizar` | `0` | Radio en píxeles para cerrar trazos punteados. |
| `invertir` | `--invertir` | `False` | Escribe alrededor de la figura en vez de dentro. |
| `umbral` | `--umbral` | auto | Umbral 0–255 de separación figura/fondo, por si el automático no te convence. |
| `aspecto` | `--aspecto` | `2.0` | Alto/ancho de un carácter en tu terminal o fuente. |

`caligrama --help` muestra lo mismo.

## Ideas para usarlo

- Una tarjeta de cumpleaños que es su foto escrita con los mensajes de todos.
- El banner de bienvenida de tu CLI con tu logo y el nombre de la herramienta.
- Un mensaje para alguien especial, con la forma de algo que solo ustedes entienden.
- Arte generativo: como cada llamada tarda milisegundos, puedes animar. El corazón de arriba sale de un bucle de 40 llamadas cambiando el tamaño de la figura y rotando el texto.

Para verlo en acción:

```bash
python examples/demo.py          # recorrido por la API
bash examples/demo.sh            # recorrido por la terminal
python examples/latido.py        # el corazón latiendo en tu terminal
```

## Cómo está hecho

El núcleo (`src/core.rs`) es Rust puro: decodifica la imagen con el crate `image`, construye la máscara, la muestrea a una rejilla de caracteres y rellena. Encima hay una capa fina de [PyO3](https://pyo3.rs) (`src/lib.rs`) que expone `dibujar`, `analizar` y el comando `caligrama`, y [maturin](https://www.maturin.rs) lo empaqueta como wheel `abi3`. El CLI también es Rust, así que se comporta igual que la API.

## Contribuir

Los issues y PRs son bienvenidos. Para montar el entorno:

```bash
git clone https://github.com/maosuarez/caligrama && cd caligrama
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
cargo test && pytest
```

Los detalles están en [CONTRIBUTING.md](https://github.com/maosuarez/caligrama/blob/main/CONTRIBUTING.md).

## Licencia

[MIT](https://github.com/maosuarez/caligrama/blob/main/LICENSE). Úsalo en lo que quieras, también en proyectos comerciales.

---

<details>
<summary><b>In English</b></summary>

**caligrama** turns any image into a *calligram*: it finds the silhouette and writes your text inside it, in reading order, returning plain text you can paste anywhere.

```bash
pip install caligrama
caligrama cat.png -w 50 -t "your poem here"
```

```python
import caligrama
print(caligrama.dibujar("cat.png", "your poem here", ancho="auto"))
info = caligrama.analizar("cat.png", "your poem here")   # capacity, per-row counts, template
```

The core is written in Rust (PyO3 + maturin) and ships as a single abi3 wheel with zero Python dependencies. Instead of a brightness threshold it detects the background from the image border and fills enclosed regions, so light-on-white subjects keep their shape. The API and CLI flags are in Spanish; the table above maps every option.

</details>
