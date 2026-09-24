# Contribuir a caligrama

Gracias por pasar por aquí. Cualquier aporte sirve: un bug con una imagen que se dibuja mal, una idea de opción nueva, un ejemplo bonito para la galería o un PR.

## Entorno

Necesitas Rust estable y Python 3.9 o más reciente.

```bash
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop          # compila e instala la extensión en el venv
```

Si usas conda y maturin se queja de que `VIRTUAL_ENV` y `CONDA_PREFIX` están definidos a la vez, haz `unset CONDA_PREFIX` en esa terminal.

## Antes de abrir un PR

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test               # núcleo en Rust
pytest -q tests          # API de Python y CLI
```

La CI corre lo mismo en Linux, macOS y Windows.

## Dónde va cada cosa

- `src/core.rs`: todo el algoritmo, sin nada de Python. Los tests del núcleo van aquí, con imágenes sintéticas.
- `src/lib.rs`: la capa de PyO3 y el CLI. Debe seguir siendo delgada.
- `caligrama.pyi`: las firmas para autocompletado. Si cambias la API, actualízala.
- `tests/`: tests de Python y del comando.

## Reglas de la casa

- La librería no puede depender de paquetes de Python. Si algo hace falta, se hace en Rust.
- La API y las opciones del CLI están en español. Mantén ese idioma en los nombres nuevos.
- Un cambio de comportamiento viene con su test.
- Anota el cambio en `CHANGELOG.md` bajo "Sin publicar".

## Publicar una versión (mantenedores)

1. Sube la versión en `Cargo.toml` y mueve las notas del CHANGELOG.
2. `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. El workflow `Release` compila las wheels, publica en PyPI con Trusted Publishing y crea el release en GitHub.
