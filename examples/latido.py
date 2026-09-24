"""Un corazón que late mientras el poema corre por dentro.

Cada fotograma es una llamada a `caligrama.dibujar`: se genera un corazón
(ecuación implícita, en BMP y sin dependencias) con un tamaño que sigue el
ritmo de un latido, y el texto se desplaza unas letras en cada paso.

    python examples/latido.py                 # animación en la terminal (Ctrl+C para salir)
    python examples/latido.py --svg latido.svg  # SVG animado (el del README)
"""

from __future__ import annotations

import argparse
import math
import re
import struct
import sys
import time
from html import escape

import caligrama

POEMA = (
    "Late despacio, late fuerte, late aunque el mundo se detenga. "
    "Cada letra que escribo aquí es un latido que te nombra, "
    "y si algún día me faltan palabras, que hable por mí este corazón hecho de texto. "
)

LADO = 240  # píxeles del lienzo
ANCHO = 56  # columnas del caligrama
FOTOGRAMAS = 40
PASO_TEXTO = 3  # letras que avanza el poema en cada fotograma


def latido(t: float) -> float:
    """Pulso doble (lub-dub) en [0, 1] para t en [0, 1)."""
    lub = math.exp(-(((t - 0.10) / 0.06) ** 2))
    dub = 0.6 * math.exp(-(((t - 0.32) / 0.07) ** 2))
    return max(lub, dub)


def corazon_bmp(escala: float) -> bytes:
    """BMP 24 bits con un corazón negro sobre blanco."""
    fila_bytes = (LADO * 3 + 3) & ~3
    datos = bytearray()
    for fila in range(LADO):  # BMP va de abajo hacia arriba
        y = (fila / LADO - 0.45) * 3.2 / escala
        linea = bytearray()
        for col in range(LADO):
            x = (col / LADO - 0.5) * 3.2 / escala
            dentro = (x * x + y * y - 1) ** 3 - x * x * y**3 <= 0
            linea += b"\x00\x00\x00" if dentro else b"\xff\xff\xff"
        datos += linea.ljust(fila_bytes, b"\x00")
    cabecera = b"BM" + struct.pack("<IHHI", 54 + len(datos), 0, 0, 54)
    info = struct.pack("<IiiHHIIiiII", 40, LADO, LADO, 1, 24, 0, len(datos), 2835, 2835, 0, 0)
    return cabecera + info + bytes(datos)


def fotogramas() -> list[list[str]]:
    salida = []
    for i in range(FOTOGRAMAS):
        escala = 0.78 + 0.22 * latido(i / FOTOGRAMAS)
        k = (i * PASO_TEXTO) % len(POEMA)
        texto = POEMA[k:] + POEMA[:k]
        # espacios="todos": el poema ya trae sus espacios; rotado, el separador sobraría.
        arte = caligrama.dibujar(corazon_bmp(escala), texto, ancho=ANCHO, espacios="todos")
        salida.append(arte.splitlines())
    return salida


def centrar(frames: list[list[str]]) -> tuple[list[list[str]], int, int]:
    """Centra cada fotograma en un lienzo común (caligrama recorta márgenes)."""
    cols = max(len(l) for f in frames for l in f)
    filas = max(len(f) for f in frames)
    centrados = []
    for f in frames:
        ancho_f = max(len(l) for l in f)
        izq = " " * ((cols - ancho_f) // 2)
        arriba = (filas - len(f)) // 2
        centrados.append([""] * arriba + [izq + l for l in f] + [""] * (filas - len(f) - arriba))
    return centrados, cols, filas


def en_terminal(frames: list[list[str]]) -> None:
    rojo, reset = "\x1b[38;5;197m", "\x1b[0m"
    sys.stdout.write("\x1b[?25l\x1b[2J")
    try:
        while True:
            for f in frames:
                sys.stdout.write("\x1b[H" + rojo + "\n".join(l.ljust(len(max(f, key=len))) for l in f) + reset)
                sys.stdout.flush()
                time.sleep(0.07)
    except KeyboardInterrupt:
        pass
    finally:
        sys.stdout.write(reset + "\x1b[?25h\n")


def a_svg(frames: list[list[str]], cols: int, filas: int) -> str:
    fs, cw, lh = 14, 8.43, 16.8  # tamaño de letra, ancho de carácter y alto de línea (≈ aspecto 2)
    pad = 28
    w, h = round(cols * cw + 2 * pad), round(filas * lh + 2 * pad + 20)
    dur = FOTOGRAMAS * 0.07
    fin = 100 / FOTOGRAMAS
    partes = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" '
        f'aria-label="Corazón que late escrito con un poema, generado con caligrama">',
        "<defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'>"
        "<stop offset='0' stop-color='#ff5f8f'/><stop offset='1' stop-color='#ff2d55'/></linearGradient></defs>",
        "<style>"
        f"text{{font:{fs}px 'DejaVu Sans Mono',Menlo,Consolas,'Liberation Mono',monospace;fill:url(#g)}}"
        f".f{{opacity:0;animation:v {dur:.2f}s step-end infinite}}"
        f"@keyframes v{{0%{{opacity:1}}{fin:.3f}%{{opacity:0}}100%{{opacity:0}}}}"
        ".pie{fill:#8b8b99;font-size:11px}"
        "</style>",
        f'<rect width="{w}" height="{h}" rx="14" fill="#14111a"/>',
    ]
    for i, f in enumerate(frames):
        partes.append(f'<g class="f" style="animation-delay:{i * 0.07:.2f}s">')
        for j, linea in enumerate(f):
            # Cada palabra con su x exacta: no depende de que el visor respete espacios.
            tramos = "".join(
                f'<tspan x="{pad + m.start() * cw:.1f}">{escape(m.group())}</tspan>'
                for m in re.finditer(r"\S+", linea)
            )
            if tramos:
                partes.append(f'<text y="{pad + (j + 1) * lh:.1f}">{tramos}</text>')
        partes.append("</g>")
    partes.append(
        f'<text class="pie" x="{w - pad}" y="{h - 14}" text-anchor="end">'
        f"{FOTOGRAMAS} fotogramas · cada uno es una llamada a caligrama.dibujar()</text>"
    )
    partes.append("</svg>")
    return "\n".join(partes)


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--svg", help="escribir un SVG animado en esta ruta en vez de animar la terminal")
    args = p.parse_args()
    frames, cols, filas = centrar(fotogramas())
    if args.svg:
        with open(args.svg, "w", encoding="utf-8") as fh:
            fh.write(a_svg(frames, cols, filas))
        print(f"{args.svg}: {len(frames)} fotogramas, {cols}×{filas}")
    else:
        en_terminal(frames)


if __name__ == "__main__":
    main()
