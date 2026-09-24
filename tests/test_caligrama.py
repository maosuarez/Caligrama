import os
import subprocess
import sys
from pathlib import Path

import pytest

import caligrama

DATOS = Path(__file__).parent / "data"
LOGO = DATOS / "python.jpeg"
POEMA = "Podrá nublarse el sol eternamente; podrá secarse en un instante el mar."


def test_ruta_str_pathlike_y_bytes_dan_lo_mismo():
    a = caligrama.dibujar(str(LOGO), POEMA, ancho=40)
    assert a == caligrama.dibujar(LOGO, POEMA, ancho=40)
    assert a == caligrama.dibujar(LOGO.read_bytes(), POEMA, ancho=40)


def test_forma_del_logo():
    lineas = caligrama.dibujar(LOGO, POEMA, ancho=80).splitlines()
    assert max(len(l) for l in lineas) <= 80
    # Mismo tamaño aproximado que la referencia ASCII de pywhatkit (80x34).
    assert 30 <= len(lineas) <= 36
    # La mitad amarilla (clara sobre fondo blanco) también se rellena.
    assert len(lineas[-3].strip()) > 20


def test_el_texto_aparece_en_orden():
    s = caligrama.dibujar(LOGO, POEMA, ancho=60, repetir=False)
    solo_texto = "".join(s.split())
    assert solo_texto.startswith("".join(POEMA.split())[:20])


def test_errores():
    with pytest.raises(OSError):
        caligrama.dibujar("no/existe.png", POEMA)
    with pytest.raises(ValueError):
        caligrama.dibujar(b"no es una imagen", POEMA)
    with pytest.raises(ValueError):
        caligrama.dibujar(LOGO, "   ")
    with pytest.raises(ValueError):
        caligrama.dibujar(LOGO, POEMA, ancho=0)


def _cli(*args, entrada=None):
    exe = Path(sys.executable).parent / "caligrama"
    return subprocess.run([exe, *args], input=entrada, capture_output=True, text=True)


def test_cli_texto_y_stdin():
    por_arg = _cli(str(LOGO), "-w", "40", "-t", POEMA)
    por_stdin = _cli(str(LOGO), "--ancho", "40", entrada=POEMA)
    assert por_arg.returncode == 0, por_arg.stderr
    assert por_arg.stdout.rstrip("\n") == caligrama.dibujar(LOGO, POEMA, ancho=40)
    assert por_stdin.stdout == por_arg.stdout


def test_cli_errores():
    r = _cli("--ancho", "x", str(LOGO))
    assert r.returncode == 2 and "--ancho" in r.stderr
    assert _cli("-h").returncode == 0


def test_ancho_auto_cabe_todo_una_vez():
    poema = POEMA * 5
    s = caligrama.dibujar(LOGO, poema, ancho="auto", repetir=False)
    assert "".join(s.split()) == "".join(poema.split())
    info = caligrama.analizar(LOGO, poema, ancho="auto")
    assert info["ancho"] == info["ancho_recomendado"]
    assert 0 <= info["sobran"] < info["letras"]
    with pytest.raises(ValueError):
        caligrama.dibujar(LOGO, POEMA, ancho="ancho")


def test_analizar():
    info = caligrama.analizar(LOGO, ancho=40)
    assert info["letras"] == sum(info["letras_por_fila"]) == info["plantilla"].count("#")
    assert info["filas"] == len(info["letras_por_fila"])
    assert info["palabras_aprox"] == info["letras"] // 6
    assert "texto_letras" not in info
    # La plantilla tiene la forma exacta del dibujo.
    relleno = caligrama.dibujar(LOGO, "x" * info["letras"], ancho=40, repetir=False)
    assert relleno.replace("x", "#") == info["plantilla"]
    por_ancho = info["letras_por_ancho"]
    assert por_ancho[30] < por_ancho[60] < por_ancho[120]
    with pytest.raises(ValueError):
        caligrama.analizar(LOGO, ancho="auto")


def test_cli_analizar():
    r = _cli("analizar", str(LOGO), "-w", "40", "-t", POEMA)
    assert r.returncode == 0, r.stderr
    assert "Caben" in r.stdout and "Tu texto:" in r.stdout and "Plantilla" in r.stdout
    # Sin texto no lee stdin (no se queda esperando).
    r = _cli("analizar", str(LOGO), entrada="")
    assert r.returncode == 0 and "Tu texto" not in r.stdout


def test_suavizar():
    base = caligrama.analizar(LOGO, ancho=40)
    suave = caligrama.analizar(LOGO, ancho=40, suavizar=3)
    assert suave["letras"] >= base["letras"]
    assert caligrama.dibujar(LOGO, POEMA, ancho=40, suavizar=0) == caligrama.dibujar(LOGO, POEMA, ancho=40)
    with pytest.raises(OverflowError):
        caligrama.dibujar(LOGO, POEMA, suavizar=-1)


def test_espacios():
    sin = caligrama.dibujar(LOGO, "Git hub", ancho=40, espacios="sin")
    assert "GithubGithub" in sin
    todos = caligrama.dibujar(LOGO, "a  b", ancho=40, espacios="todos")
    assert "a  ba  b" in todos
    info = caligrama.analizar(LOGO, "a  b\n", espacios="todos")
    assert info["texto_letras"] == 4
    with pytest.raises(ValueError):
        caligrama.dibujar(LOGO, POEMA, espacios="algunos")
