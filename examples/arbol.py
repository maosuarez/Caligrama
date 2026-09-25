"""Un árbol escrito con un poema, cada letra con el color de la imagen.

    python examples/arbol.py                        # en la terminal (color ANSI de 24 bits)
    python examples/arbol.py --svg assets/arbol.svg # SVG (el del README)

`color=True` pinta cada letra con el color medio de la imagen en su celda: la
copa sale verde y el tronco café sin decirle dónde está cada cosa.
"""

import argparse
import sys
from pathlib import Path

import caligrama

IMAGEN = Path(__file__).resolve().parent.parent / "assets" / "arbol.png"
POEMA = """
Quiero crecer contigo,
como crece este árbol:
despacio,
con raíces fuertes,
con ramas que buscan el cielo
y hojas que guardan
cada momento que vivimos.
"""


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--svg", help="escribir un SVG en esta ruta en vez de imprimir")
    p.add_argument("-w", "--ancho", type=int, default=70)
    args = p.parse_args()

    # Sin espacios la copa queda sólida y se parece más al árbol.
    dibujo = caligrama.dibujar(IMAGEN, POEMA, ancho=args.ancho, espacios="sin", color=True)
    if args.svg:
        with open(args.svg, "w", encoding="utf-8") as fh:
            fh.write(caligrama.a_svg(dibujo, fondo="#14111a"))
        print(args.svg)
    else:
        print(dibujo)


if __name__ == "__main__":
    main()
