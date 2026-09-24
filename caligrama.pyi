"""Escribe mensajes y poemas dentro de la silueta de una imagen."""

import os
from typing import Literal, TypedDict, Union

from typing_extensions import NotRequired

__version__: str

Imagen = Union[str, "os.PathLike[str]", bytes]
Ancho = Union[int, Literal["auto"]]
Espacios = Literal["normal", "sin", "todos"]

class Analisis(TypedDict):
    ancho: int
    filas: int
    letras: int
    palabras_aprox: int
    letras_por_fila: list[int]
    tramos: int
    tramo_min: int
    tramo_max: int
    plantilla: str
    letras_por_ancho: dict[int, int]
    texto_letras: NotRequired[int]
    ancho_recomendado: NotRequired[Union[int, None]]
    sobran: NotRequired[int]

def dibujar(
    imagen: Imagen,
    texto: str,
    ancho: Ancho = 60,
    aspecto: float = 2.0,
    repetir: bool = True,
    espacios: Espacios = "normal",
    umbral: Union[int, None] = None,
    invertir: bool = False,
    huecos: bool = False,
    suavizar: int = 0,
) -> str:
    """Escribe `texto` dentro de la silueta de `imagen` y devuelve el dibujo."""

def analizar(
    imagen: Imagen,
    texto: Union[str, None] = None,
    ancho: Ancho = 60,
    aspecto: float = 2.0,
    espacios: Espacios = "normal",
    umbral: Union[int, None] = None,
    invertir: bool = False,
    huecos: bool = False,
    suavizar: int = 0,
) -> Analisis:
    """Examina la silueta sin escribir: capacidad, filas, plantilla y ajuste del texto."""

def cli() -> int:
    """Punto de entrada del comando `caligrama`."""
