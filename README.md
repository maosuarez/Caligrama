<div align="center">

<h1><img src="https://raw.githubusercontent.com/maosuarez/caligrama/main/assets/titulo.svg" alt="caligrama" width="100%"></h1>

**Dale una imagen y un poema. Te devuelve el poema con la forma de la imagen.**

[![PyPI](https://img.shields.io/pypi/v/caligrama?color=ff2d55&label=pypi&cacheSeconds=3600)](https://pypi.org/project/caligrama/)
[![Python 3.9+](https://img.shields.io/badge/python-3.9%2B-3776ab)](https://pypi.org/project/caligrama/)
[![CI](https://github.com/maosuarez/caligrama/actions/workflows/ci.yml/badge.svg)](https://github.com/maosuarez/caligrama/actions/workflows/ci.yml)
[![Licencia MIT](https://img.shields.io/badge/licencia-MIT-blue)](https://github.com/maosuarez/caligrama/blob/main/LICENSE)
[![Hecho en Rust](https://img.shields.io/badge/núcleo-Rust-orange)](https://github.com/maosuarez/caligrama/tree/main/src)
[![Documentación](https://img.shields.io/badge/docs-ES%20%7C%20EN-ff2d55)](https://www.maosuarez.com/docs/caligrama)

<img src="https://raw.githubusercontent.com/maosuarez/caligrama/main/assets/latido.svg" alt="Un corazón que late, escrito con un poema que corre por dentro. Generado con caligrama." width="420">

<sub>El título y este corazón los genera la propia librería con <code>caligrama animar</code>. Código en <a href="https://github.com/maosuarez/caligrama/blob/main/examples/latido.py"><code>examples/latido.py</code></a>.</sub>

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

Y si se lo pides, cada letra toma el color que tiene la imagen en ese punto:

<div align="center">
<table>
<tr>
<td align="center"><img src="https://raw.githubusercontent.com/maosuarez/caligrama/main/assets/arbol.png" alt="Ilustración de un árbol con copa verde y tronco café" width="180"><br><sub>la imagen</sub></td>
<td align="center"><img src="https://raw.githubusercontent.com/maosuarez/caligrama/main/assets/arbol.svg" alt="El mismo árbol escrito con un poema; la copa sale verde y el tronco café" width="560"><br><sub>el caligrama</sub></td>
</tr>
</table>
</div>

<p align="center"><sub><code>caligrama arbol.png -w 70 --espacios sin --colores imagen -f poema.txt</code> · código en <a href="https://github.com/maosuarez/caligrama/blob/main/examples/arbol.py"><code>examples/arbol.py</code></a></sub></p>

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

## Animaciones, en la terminal o en SVG

El título de este README sale de un solo comando:

```bash
caligrama animar assets/titulo.png -t caligrama -w 160 --espacios sin --con-huecos \
    --colores "#ff2d55,#ff8a3d,#b44dff" --svg titulo.svg
```

Sin `--svg`, la animación se reproduce en la terminal hasta que pulses Ctrl+C. Hay dos movimientos, y se pueden combinar:

- `--paso N`: el texto avanza N letras por fotograma y parece correr por dentro de la figura. Por defecto, `animar` calcula cuántos fotogramas hacen falta para que el texto dé la vuelta completa, así que el bucle no tiene salto.
- `--latido F`: la figura late con un pulso doble y se encoge hasta el F·100 % de su tamaño entre latidos.

Desde Python son tres funciones que encajan entre sí:

```python
fotos = caligrama.animar("corazon.png", poema, paso=3, latido=0.22, ancho=56)   # list[str]
caligrama.reproducir(fotos, intervalo=0.07, colores=["#ff5f8f", "#ff2d55"])        # en la terminal
svg = caligrama.a_svg(fotos, intervalo=0.07, fondo="#14111a")                       # para tu README o tu web
```

Los fotogramas son texto normal, todos del mismo tamaño y alineados, así que también puedes llevarlos a un GIF, a una web o a donde quieras. Los colores de la terminal usan ANSI de 24 bits: funcionan en Windows Terminal, iTerm2, GNOME Terminal y en casi cualquier terminal moderna.

## Los colores de la imagen

Con `color=True` (o `--colores imagen`) cada letra se pinta con el color que tiene la imagen en ese punto. Así sale el árbol del principio: nadie le dijo dónde estaba la copa ni dónde el tronco. El color de cada celda es el promedio de sus píxeles de figura, así que el blanco del fondo no aclara los bordes.

```python
print(caligrama.dibujar("arbol.png", poema, ancho=70, espacios="sin", color=True))
svg = caligrama.a_svg(caligrama.animar("arbol.png", poema, fotogramas=24, color=True), fondo="#111")
```

```bash
caligrama arbol.png -f poema.txt -w 70 --colores imagen
caligrama animar arbol.png -f poema.txt --colores imagen --svg arbol.svg
```

El resultado sigue siendo un `str`, con los colores en ANSI; `a_svg` y `reproducir` los respetan tal cual. Si además pasas un degradado con `colores=`, el degradado manda.

Dos cosas a tener en cuenta. En el SVG cada letra de color lleva su propia etiqueta, así que una animación larga pesa: limita los fotogramas si va a un README. Y una figura muy oscura (un gato negro) casi no se ve sobre una terminal oscura; es el color real de la imagen.

## Frente a otras herramientas

Puse el árbol de arriba y el mismo poema en todo lo que encontré para dibujar imágenes con texto en la terminal, a 70 columnas:

| | Qué dibuja | Tiempo | Import | Paquetes | Palabras que se leen | Colores |
|---|---|---:|---:|---:|---:|---:|
| **caligrama** `color=True` | tu texto, con la forma y los colores de la imagen | 2,1 ms | 1 ms | 1 | 80 % | 1161 |
| Pillow + rich, a mano | tu texto sin espacios, con 2 colores fijos | 13,7 ms | 40 ms | 5 | 17 % | 2 |
| [ascii_magic](https://pypi.org/project/ascii-magic/) | símbolos según el brillo, 16 colores | 22,1 ms | 42 ms | 2 | 0 % | 5 |
| [pywhatkit](https://pypi.org/project/pywhatkit/) | símbolos según el brillo, siempre a 80 columnas | 1,6 ms | 904 ms | 27 | 0 % | 0 |
| [viu](https://github.com/atanunq/viu) | la foto, dos píxeles por celda con `▄` | 3,2 ms¹ | — | binario | 0 % | 3007 |
| [catimg](https://github.com/posva/catimg) | la foto, dos píxeles por celda con `▀` | 3,3 ms¹ | — | binario | 0 % | 3184 |

<sub>Mediana de 5 ejecuciones en un portátil con WSL2 y Python 3.11; el import se mide aparte, en un proceso nuevo. ¹ Incluye arrancar el programa, que es como se usan. "Palabras que se leen" es el porcentaje de palabras del dibujo que son palabras enteras del poema. "Paquetes" cuenta todo lo que se instala, dependencias y el propio paquete incluidos.</sub>

Si lo que quieres es ver una foto en la terminal, viu y catimg ganan sin discusión: pintan dos píxeles de color por celda y ningún dibujo hecho con letras se les acerca. Pero no llevan mensaje, y si copias su salida como texto plano te queda un rectángulo de bloques. ascii_magic y pywhatkit cambian el brillo por símbolos: la imagen se reconoce y el texto no existe.

Hacerlo a mano con Pillow sirve hasta que la imagen tiene ruido de JPEG, un amarillo claro sobre blanco o un hueco en medio; ahí empiezan los ajustes que caligrama ya trae (la sección siguiente cuenta cuáles). En la tabla también se ve lo otro: sin espacios las palabras se pegan y casi nada se lee. caligrama con `espacios="sin"`, como el árbol de arriba, baja al 13 %. Es el precio de una figura más sólida.

Para repetirlo con tus imágenes: `python bench/benchmark.py`. Deja en `bench/resultados/` un informe HTML con todos los dibujos lado a lado.

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
| `desfase` | `--desfase` | `0` | Empieza a escribir el texto N letras más adelante. |
| `color` | `--colores imagen` | `False` | Pinta cada letra con el color de la imagen en su posición (ANSI de 24 bits). |
| `colores` | `--colores` | sin color | Degradado horizontal en hexadecimal, en la terminal y en el SVG. |
| (`a_svg`) | `--svg` | | Guarda un SVG en vez de imprimir (animado con `animar`). |

Solo para animar: `fotogramas`/`--fotogramas`, `paso`/`--paso`, `latido`/`--latido`, `intervalo`/`--intervalo` y, en la terminal, `veces`/`--veces`.

`caligrama --help` muestra lo mismo.

La [documentación completa](https://www.maosuarez.com/docs/caligrama) (en español e inglés) explica cada opción con ejemplos, recetas (web, Jupyter, tarjetas, banners), la referencia del CLI y qué hacer cuando la silueta no sale bien. *Full documentation in English: [maosuarez.com/docs/caligrama?lang=en](https://www.maosuarez.com/docs/caligrama?lang=en).*

## Ideas para usarlo

- Una tarjeta de cumpleaños que es su foto escrita con los mensajes de todos.
- El banner de bienvenida de tu CLI con tu logo y el nombre de la herramienta.
- Un mensaje para alguien especial, con la forma de algo que solo ustedes entienden.
- Una pantalla de carga para tu CLI con tu logo latiendo mientras el texto corre por dentro.

Para verlo en acción:

```bash
python examples/demo.py          # recorrido por la API
bash examples/demo.sh            # recorrido por la terminal
python examples/latido.py        # el corazón latiendo en tu terminal
python examples/titulo.py        # el título de este README, animado en tu terminal
python examples/arbol.py         # el árbol a color del principio
```

## Cómo está hecho

El núcleo (`src/core.rs`) es Rust puro: decodifica la imagen con el crate `image`, construye la máscara, la muestrea a una rejilla de caracteres y rellena. Las animaciones, el SVG y los colores viven en `src/animacion.rs`, también en Rust. Encima hay una capa fina de [PyO3](https://pyo3.rs) (`src/lib.rs`) que expone `dibujar`, `analizar`, `animar`, `a_svg`, `reproducir` y el comando `caligrama`, y [maturin](https://www.maturin.rs) lo empaqueta como wheel `abi3`. El CLI también es Rust, así que se comporta igual que la API.

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
frames = caligrama.animar("heart.png", "your poem ", paso=2, latido=0.2)
caligrama.reproducir(frames)                               # animate in the terminal
open("heart.svg", "w").write(caligrama.a_svg(frames))       # or export an animated SVG
print(caligrama.dibujar("tree.png", "your poem", color=True)) # each letter takes the image's color
```

Compared with image viewers (viu, catimg) and ASCII-art tools (ascii_magic, pywhatkit) on the same image, caligrama is the one that writes your text legibly: 80 % of words intact, against 17 % for a hand-written Pillow + rich script and 0 % for the rest. It has no dependencies and imports in about 1 ms. Viewers are more faithful to the photo; they just can't carry a message. Run `python bench/benchmark.py` to reproduce.

The core is written in Rust (PyO3 + maturin) and ships as a single abi3 wheel with zero Python dependencies. Instead of a brightness threshold it detects the background from the image border and fills enclosed regions, so light-on-white subjects keep their shape. The API and CLI flags are in Spanish; the table above maps every option.

</details>
