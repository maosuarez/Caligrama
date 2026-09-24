"""El título del README: la palabra CALIGRAMA escrita con la palabra caligrama.

    python examples/titulo.py            # imprime el título en la terminal
    python examples/titulo.py --svg assets/titulo.svg

Cada fotograma rota el texto una letra; como "caligrama" tiene 9 letras, a los
9 fotogramas el dibujo vuelve a empezar y el bucle no tiene costura.
"""

from __future__ import annotations

import argparse
import re
import sys
from html import escape
from pathlib import Path

import caligrama

IMAGEN = Path(__file__).resolve().parent.parent / "assets" / "titulo.png"
PALABRA = "caligrama"
ANCHO = 160
PASO = 0.14  # segundos por fotograma


def fotograma(k: int) -> list[str]:
    texto = PALABRA[k:] + PALABRA[:k]
    lineas = caligrama.dibujar(IMAGEN, texto, ancho=ANCHO, espacios="sin", huecos=True).splitlines()
    # La sombra de la tipografía son líneas finas que asoman bajo las letras y
    # dejan una última fila casi vacía; se quita para que el título respire.
    llenas = [len(l.replace(" ", "")) for l in lineas]
    while llenas and llenas[-1] < 0.4 * max(llenas):
        lineas.pop()
        llenas.pop()
    return lineas


def a_svg(frames: list[list[str]]) -> str:
    cols = max(len(l) for f in frames for l in f)
    filas = max(len(f) for f in frames)
    fs, cw, lh = 14, 8.43, 16.8
    w, h = round(cols * cw), round(filas * lh + 6)
    dur = len(frames) * PASO
    fin = 100 / len(frames)
    partes = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" '
        f'role="img" aria-label="caligrama">',
        "<defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='0'>"
        "<stop offset='0' stop-color='#ff2d55'/><stop offset='.5' stop-color='#ff8a3d'/>"
        "<stop offset='1' stop-color='#b44dff'/></linearGradient></defs>",
        "<style>"
        f"text{{font:bold {fs}px 'DejaVu Sans Mono',Menlo,Consolas,'Liberation Mono',monospace;fill:url(#g)}}"
        f".f{{opacity:0;animation:v {dur:.2f}s step-end infinite}}"
        f"@keyframes v{{0%{{opacity:1}}{fin:.3f}%{{opacity:0}}100%{{opacity:0}}}}"
        "</style>",
    ]
    for i, f in enumerate(frames):
        partes.append(f'<g class="f" style="animation-delay:{i * PASO:.2f}s">')
        for j, linea in enumerate(f):
            # Cada tramo con su x exacta: no depende de que el visor respete espacios.
            tramos = "".join(
                f'<tspan x="{m.start() * cw:.1f}">{escape(m.group())}</tspan>' for m in re.finditer(r"\S+", linea)
            )
            if tramos:
                partes.append(f'<text y="{(j + 1) * lh:.1f}">{tramos}</text>')
        partes.append("</g>")
    partes.append("</svg>")
    return "\n".join(partes)


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--svg", help="escribir el SVG animado en esta ruta")
    args = p.parse_args()
    if not args.svg:
        print("\n".join(fotograma(0)))
        return
    frames = [fotograma(k) for k in range(len(PALABRA))]
    Path(args.svg).write_text(a_svg(frames), encoding="utf-8")
    print(f"{args.svg}: {len(frames)} fotogramas")


if __name__ == "__main__":
    main()
