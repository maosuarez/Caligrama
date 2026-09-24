"""Recorrido por la API de caligrama. Ejecuta: python examples/demo.py"""

import sys
from pathlib import Path

import caligrama

# En Windows la salida redirigida usa cp1252, que no tiene "─" ni todas las letras.
sys.stdout.reconfigure(encoding="utf-8")

LOGO = Path(__file__).resolve().parent.parent / "tests" / "data" / "python.jpeg"

POEMA = """Podrá nublarse el sol eternamente;
podrá secarse en un instante el mar;
podrá romperse el eje de la tierra
como un débil cristal."""


def titulo(t: str) -> None:
    print(f"\n\x1b[1m── {t} " + "─" * (60 - len(t)) + "\x1b[0m\n")


titulo("1. Lo básico: imagen + texto -> str")
print(caligrama.dibujar(LOGO, "Hola mundo", ancho=40))

titulo("2. Antes de escribir: ¿cuánto texto cabe?")
info = caligrama.analizar(LOGO, POEMA, ancho=40)
print(f"A {info['ancho']} columnas caben {info['letras']} letras (~{info['palabras_aprox']} palabras).")
print(f"Tu poema tiene {info['texto_letras']} letras; cabe completo desde {info['ancho_recomendado']} columnas.")
print("Capacidad por ancho:", info["letras_por_ancho"])

titulo('3. ancho="auto": el poema entero, una sola vez')
print(caligrama.dibujar(LOGO, POEMA, ancho="auto", repetir=False))

titulo('4. espacios="sin": la silueta más sólida')
print(caligrama.dibujar(LOGO, "Python", ancho=40, espacios="sin"))

titulo("5. invertir=True: escribir alrededor de la figura")
print(caligrama.dibujar(LOGO, "fondo ", ancho=40, invertir=True, espacios="todos"))

titulo("6. También acepta bytes (útil con requests, bases de datos, etc.)")
print(caligrama.dibujar(LOGO.read_bytes(), "bytes", ancho=24, espacios="sin"))
