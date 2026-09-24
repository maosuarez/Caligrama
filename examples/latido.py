"""Un corazón que late mientras el poema corre por dentro.

    python examples/latido.py                   # en la terminal (Ctrl+C para salir)
    python examples/latido.py --svg latido.svg  # SVG animado (el del README)

La figura es un corazón generado aquí mismo (ecuación implícita, en BMP y sin
dependencias). Todo lo demás lo hace caligrama: `animar` con `latido` lo
encoge y lo agranda con un pulso doble, y con `paso` el poema avanza 3 letras
por fotograma.
"""

import argparse
import struct
import sys

import caligrama

POEMA = (
    "Late despacio, late fuerte, late aunque el mundo se detenga. "
    "Cada letra que escribo aquí es un latido que te nombra, "
    "y si algún día me faltan palabras, que hable por mí este corazón hecho de texto. "
)
LADO = 240


def corazon_bmp() -> bytes:
    """BMP 24 bits con un corazón negro sobre blanco."""
    fila_bytes = (LADO * 3 + 3) & ~3
    datos = bytearray()
    for fila in range(LADO):  # BMP va de abajo hacia arriba
        y = (fila / LADO - 0.45) * 3.2
        linea = bytearray()
        for col in range(LADO):
            x = (col / LADO - 0.5) * 3.2
            dentro = (x * x + y * y - 1) ** 3 - x * x * y**3 <= 0
            linea += b"\x00\x00\x00" if dentro else b"\xff\xff\xff"
        datos += linea.ljust(fila_bytes, b"\x00")
    cabecera = b"BM" + struct.pack("<IHHI", 54 + len(datos), 0, 0, 54)
    info = struct.pack("<IiiHHIIiiII", 40, LADO, LADO, 1, 24, 0, len(datos), 2835, 2835, 0, 0)
    return cabecera + info + bytes(datos)


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--svg", help="escribir un SVG animado en esta ruta en vez de animar la terminal")
    args = p.parse_args()

    # espacios="todos": el poema ya trae sus espacios, no hace falta separador.
    fotogramas = caligrama.animar(
        corazon_bmp(), POEMA, fotogramas=40, paso=3, latido=0.22, ancho=56, espacios="todos"
    )
    colores = ["#ff5f8f", "#ff2d55"]
    if args.svg:
        svg = caligrama.a_svg(fotogramas, intervalo=0.07, colores=colores, fondo="#14111a")
        with open(args.svg, "w", encoding="utf-8") as fh:
            fh.write(svg)
        print(f"{args.svg}: {len(fotogramas)} fotogramas")
    else:
        caligrama.reproducir(fotogramas, intervalo=0.07, colores=colores)


if __name__ == "__main__":
    main()
