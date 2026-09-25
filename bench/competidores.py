"""Las soluciones que se comparan. Cada una recibe (imagen, poema, ancho) y
devuelve un str (con color ANSI si lo tiene).

"A mano" son dos scripts típicos con Pillow (máscara por distancia al color de
las esquinas, umbral fijo) más rich o pyfiglet: lo que uno escribiría sin
caligrama.
"""

import io
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

ASPECTO = 2.0


@dataclass
class Competidor:
    nombre: str
    familia: str  # "densidad", "visor", "textura" o "poema"
    paquetes: list[str]  # distribuciones de PyPI que hay que instalar
    modulos: list[str]  # lo que se importa (para medir el tiempo de import)
    fn: Callable[[Path, str, int], str]
    nota: str = ""
    opciones: dict = field(default_factory=dict)
    # Programas externos: se mide el proceso entero (arranque incluido).
    binario: str | None = None
    instalar: str = ""


# --- Hechos a mano con Pillow ------------------------------------------------


def _mascara_pil(path, width):
    """Máscara a mano: fondo = promedio de las 4 esquinas, umbral fijo 45."""
    from PIL import Image

    img = Image.open(path).convert("RGB")
    esquinas = [
        img.getpixel((0, 0)),
        img.getpixel((img.width - 1, 0)),
        img.getpixel((0, img.height - 1)),
        img.getpixel((img.width - 1, img.height - 1)),
    ]
    bg = tuple(sum(p[i] for p in esquinas) // 4 for i in range(3))
    height = max(1, int(img.height / img.width * width / ASPECTO))
    img = img.resize((width, height))
    return [
        [sum(abs(c - b) for c, b in zip(img.getpixel((x, y)), bg)) > 45 for x in range(width)]
        for y in range(height)
    ]


def pil_rich(path, poema, ancho):
    from rich.console import Console
    from rich.text import Text

    mask = _mascara_pil(path, ancho)
    texto = poema.replace(" ", "").replace("\n", "")
    buf = io.StringIO()
    console = Console(file=buf, force_terminal=True, color_system="truecolor", width=ancho + 10)
    pos = 0
    for y, row in enumerate(mask):
        salida = Text()
        for pertenece in row:
            if pertenece:
                estilo = "bold #8B4513" if y > len(mask) * 0.68 else "bold green"
                salida.append(texto[pos % len(texto)], style=estilo)
                pos += 1
            else:
                salida.append(" ")
        console.print(salida)
    return buf.getvalue()


def pil_pyfiglet(path, poema, ancho):
    import pyfiglet

    mask = _mascara_pil(path, ancho)
    lineas = pyfiglet.Figlet(font="small", width=ancho).renderText("AMOR").splitlines()
    filas = []
    for y, row in enumerate(mask):
        linea = lineas[y % len(lineas)] or " "
        while len(linea) < ancho:
            linea += linea
        filas.append("".join(linea[x] if b else " " for x, b in enumerate(row)))
    return "\n".join(filas)


# --- Librerías de arte ASCII por densidad -----------------------------------


def pywhatkit_ascii(path, poema, ancho):
    import contextlib
    import os

    with open(os.devnull, "w") as nul, contextlib.redirect_stderr(nul), contextlib.redirect_stdout(nul):
        import pywhatkit  # importa pyautogui y avisa por Xlib
    with tempfile.TemporaryDirectory() as d:
        return pywhatkit.image_to_ascii_art(str(path), str(Path(d) / "x"))


def ascii_magic_color(path, poema, ancho):
    import ascii_magic

    import contextlib

    # to_ascii() es monocromo; to_terminal() trae el color pero además lo imprime.
    arte = ascii_magic.AsciiArt.from_image(str(path))
    with contextlib.redirect_stdout(io.StringIO()):
        return arte.to_terminal(columns=ancho, width_ratio=ASPECTO)


# --- Visores de imágenes en la terminal (medio bloque "▀" con color) ------------


def _ejecutar(*args):
    import os
    import subprocess

    env = {**os.environ, "COLORTERM": "truecolor", "TERM": "xterm-256color"}
    r = subprocess.run(args, capture_output=True, text=True, env=env, check=True)
    return r.stdout


def viu(path, poema, ancho):
    # -b: bloques de texto aunque la terminal soporte Kitty/iTerm/Sixel.
    return _ejecutar("viu", "-b", "-w", str(ancho), str(path))


def catimg(path, poema, ancho):
    # -r 2: medio bloque. Con -r 2, -w cuenta medias columnas: se pide el doble.
    # Al final solo vuelve a mostrar el cursor; esa línea no es parte del dibujo.
    salida = _ejecutar("catimg", "-r", "2", "-w", str(2 * ancho), str(path))
    return salida.removesuffix("\x1b[?25h").rstrip("\n")


# --- caligrama ---------------------------------------------------------------


def _caligrama(**opciones):
    def fn(path, poema, ancho):
        import caligrama

        return caligrama.dibujar(path, poema, ancho=ancho, **opciones)

    return fn


def todos() -> list[Competidor]:
    return [
        Competidor(
            "pywhatkit", "densidad", ["pywhatkit"], ["pywhatkit"], pywhatkit_ascii,
            "Rampa de símbolos por brillo; siempre 80 columnas, no usa el poema.",
        ),
        Competidor(
            "ascii_magic", "densidad", ["ascii_magic"], ["ascii_magic"], ascii_magic_color,
            "Rampa de símbolos con color; no usa el poema.",
        ),
        Competidor(
            "viu", "visor", [], [], viu,
            "Visor de imágenes (Rust): pinta píxeles con medio bloque y color de 24 bits.",
            binario="viu", instalar="cargo install viu",
        ),
        Competidor(
            "catimg", "visor", [], [], catimg,
            "Visor de imágenes (C): pinta píxeles con medio bloque y color.",
            binario="catimg", instalar="sudo apt install catimg",
        ),
        Competidor(
            "PIL + pyfiglet (a mano)", "textura", ["pillow", "pyfiglet"], ["PIL.Image", "pyfiglet"],
            pil_pyfiglet, "Letras grandes de figlet recortadas por la máscara.",
        ),
        Competidor(
            "PIL + rich (a mano)", "poema", ["pillow", "rich"], ["PIL.Image", "rich.console"],
            pil_rich, "Máscara a mano; 2 colores fijos (verde / café bajo el 68 %).",
        ),
        Competidor(
            "caligrama", "poema", ["caligrama"], ["caligrama"], _caligrama(),
            "Valores por defecto.",
        ),
        Competidor(
            "caligrama sin espacios", "poema", ["caligrama"], ["caligrama"],
            _caligrama(espacios="sin", color=True), "Como 03_caligrama.py, con color de la imagen.",
            {"espacios": "sin", "color": True},
        ),
        Competidor(
            "caligrama color", "poema", ["caligrama"], ["caligrama"], _caligrama(color=True),
            "Palabras separadas y color de la imagen.", {"color": True},
        ),
    ]
