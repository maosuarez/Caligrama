# Changelog

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

## [0.1.0] - 2026-09-24

Primera versión.

### Añadido

- `caligrama.dibujar(imagen, texto, ...)`: escribe el texto dentro de la silueta y devuelve un `str`. Acepta ruta, `PathLike` o bytes; lee PNG, JPEG, GIF, BMP y WebP.
- `caligrama.analizar(imagen, texto=None, ...)`: capacidad en letras y palabras, letras por fila, tramos, plantilla y, si se pasa texto, el ancho donde cabe completo.
- Comando `caligrama` con los subcomandos de dibujo y `analizar`.
- Detección de figura por distancia al color del borde, con umbral automático ajustado al ruido, y uso del canal alfa si existe.
- Relleno de huecos interiores (`huecos=True` para conservarlos).
- `ancho="auto"`, `espacios="normal"|"sin"|"todos"`, `suavizar`, `invertir`, `umbral`, `aspecto`, `repetir`.
- Wheel `abi3` para Python 3.9+ sin dependencias de Python.

[Sin publicar]: https://github.com/maosuarez/caligrama/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/maosuarez/caligrama/releases/tag/v0.1.0
