"""El título del README: la palabra CALIGRAMA escrita con la palabra caligrama.

    python examples/titulo.py                         # animado en la terminal (Ctrl+C para salir)
    python examples/titulo.py --svg assets/titulo.svg

Es lo mismo que:

    caligrama animar assets/titulo.png -t caligrama -w 160 --espacios sin --con-huecos \\
        --intervalo 0.14 --colores "#ff2d55,#ff8a3d,#b44dff" --svg assets/titulo.svg

"caligrama" tiene 9 letras, así que `animar` genera 9 fotogramas y el bucle no
tiene costura. `assets/titulo.png` es la tipografía original sin las líneas
finas de su sombra (cierre morfológico y 12 px menos abajo), que ensuciaban la
última fila.
"""

import argparse
import sys
from pathlib import Path

import caligrama

IMAGEN = Path(__file__).resolve().parent.parent / "assets" / "titulo.png"
COLORES = ["#ff2d55", "#ff8a3d", "#b44dff"]

sys.stdout.reconfigure(encoding="utf-8")
p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
p.add_argument("--svg", help="escribir el SVG animado en esta ruta")
args = p.parse_args()

fotogramas = caligrama.animar(IMAGEN, "caligrama", ancho=160, espacios="sin", huecos=True)
if args.svg:
    Path(args.svg).write_text(caligrama.a_svg(fotogramas, intervalo=0.14, colores=COLORES), encoding="utf-8")
    print(f"{args.svg}: {len(fotogramas)} fotogramas")
else:
    caligrama.reproducir(fotogramas, intervalo=0.14, colores=COLORES)
