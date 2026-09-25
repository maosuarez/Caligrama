# Changelog

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

## [0.3.1] - 2026-09-24

### Añadido

- `examples/arbol.py` y `assets/arbol.svg`: el árbol a color del README.
- `bench/`: benchmark contra pywhatkit, ascii_magic, viu, catimg y scripts hechos a mano, con informe HTML.

### Cambiado

- README: árbol a color al principio y sección "Frente a otras herramientas" con los resultados del benchmark.

## [0.3.0] - 2026-09-24

### Añadido

- `color=True` en `dibujar` y `animar` (`--colores imagen` en el CLI): cada letra lleva, en ANSI de 24 bits, el color medio de la imagen en su celda.
- `a_svg` y `reproducir` respetan el color ANSI que traigan los fotogramas; en el SVG cada tramo de palabra lleva su `fill`.

## [0.2.0] - 2026-09-24

### Añadido

- `caligrama.animar(...)`: fotogramas alineados en los que el texto fluye (`paso`) y la figura late (`latido`). Sin `fotogramas`, calcula los justos para un bucle sin costura.
- `caligrama.a_svg(...)`: exporta uno o varios fotogramas a SVG, animado si hay varios, con degradado y fondo opcional.
- `caligrama.reproducir(...)`: anima los fotogramas en la terminal, con color ANSI de 24 bits opcional; Ctrl+C lo detiene y restaura el cursor.
- Subcomando `caligrama animar` y opciones `--svg`, `--colores`, `--fondo`, `--tamano`, `--paso`, `--latido`, `--fotogramas`, `--intervalo`, `--veces`.
- `desfase` / `--desfase` en `dibujar` para empezar el texto N letras más adelante.

### Cambiado

- Los errores del CLI muestran un aviso corto en lugar de la ayuda completa.
- `assets/titulo.png` ahora no trae las líneas finas de la sombra, así que el título sale de un solo comando.
- `examples/latido.py` y `examples/titulo.py` usan la nueva API y ya no generan el SVG por su cuenta.

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

[Sin publicar]: https://github.com/maosuarez/caligrama/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/maosuarez/caligrama/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/maosuarez/caligrama/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/maosuarez/caligrama/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/maosuarez/caligrama/releases/tag/v0.1.0
