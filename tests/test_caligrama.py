import os
import re
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
    # caligrama lee y escribe UTF-8; en Windows el locale por defecto es cp1252.
    return subprocess.run([exe, *args], input=entrada, capture_output=True, encoding="utf-8")


def test_cli_texto_y_stdin():
    por_arg = _cli(str(LOGO), "-w", "40", "-t", POEMA)
    por_stdin = _cli(str(LOGO), "--ancho", "40", entrada=POEMA)
    assert por_arg.returncode == 0, por_arg.stderr
    assert por_arg.stdout.rstrip("\n") == caligrama.dibujar(LOGO, POEMA, ancho=40)
    assert por_stdin.stdout == por_arg.stdout


def test_cli_errores():
    r = _cli("--ancho", "x", str(LOGO))
    assert r.returncode == 2 and "ancho" in r.stderr and "--help" in r.stderr
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


def test_desfase_hace_fluir_el_texto():
    a = caligrama.dibujar(LOGO, "abc", ancho=30)
    b = caligrama.dibujar(LOGO, "abc", ancho=30, desfase=1)
    assert a.lstrip()[0] == "a" and b.lstrip()[0] == "b"
    # "abc" + separador: 4 letras de ciclo.
    assert caligrama.dibujar(LOGO, "abc", ancho=30, desfase=4) == a


def test_animar():
    fotos = caligrama.animar(LOGO, "caligrama", ancho=40, espacios="sin")
    assert len(fotos) == 9  # bucle sin costura: una vuelta completa del texto
    assert len({f.count("\n") for f in fotos}) == 1
    assert fotos[0] == caligrama.dibujar(LOGO, "caligrama", ancho=40, espacios="sin")
    assert fotos[1] == caligrama.dibujar(LOGO, "caligrama", ancho=40, espacios="sin", desfase=1)
    latiendo = caligrama.animar(LOGO, "x", ancho=30, paso=0, latido=0.3, fotogramas=10)
    assert len(latiendo) == 10 and len(set(latiendo)) > 1
    with pytest.raises(ValueError):
        caligrama.animar(LOGO, "x", latido=1.5)


def test_a_svg():
    fotos = caligrama.animar(LOGO, "ab", ancho=20, fotogramas=3)
    svg = caligrama.a_svg(fotos, colores=["#000"], fondo="#fff")
    assert svg.startswith("<svg") and svg.count('<g class="f"') == 3
    assert "#000000" in svg and "#ffffff" in svg
    assert "@keyframes" not in caligrama.a_svg(fotos[0])
    with pytest.raises(ValueError):
        caligrama.a_svg(fotos, colores=["red"])


def test_color_de_la_imagen(tmp_path):
    plano = caligrama.dibujar(LOGO, "abc", ancho=30)
    color = caligrama.dibujar(LOGO, "abc", ancho=30, color=True)
    assert re.sub(r"\x1b\[[\d;]*m", "", color) == plano
    tonos = set(re.findall(r"\x1b\[38;2;(\d+);(\d+);(\d+)m", color))
    # Logo azul y amarillo: hay tonos de los dos.
    assert any(int(b) > int(r) for r, _, b in tonos)
    assert any(int(r) > int(b) for r, _, b in tonos)
    svg = caligrama.a_svg(color)
    assert "\x1b" not in svg and svg.count("fill=") > 2
    r = _cli(str(LOGO), "-w", "30", "-t", "abc", "--colores", "imagen")
    assert r.returncode == 0 and r.stdout.rstrip("\n") == color
    destino = tmp_path / "c.svg"
    r = _cli("animar", str(LOGO), "-w", "20", "-t", "ab", "--colores", "imagen", "--svg", str(destino))
    assert r.returncode == 0, r.stderr
    assert 'fill="#' in destino.read_text(encoding="utf-8")


def test_reproducir(capfd):
    caligrama.reproducir(["ab", "cd"], intervalo=0, veces=1)
    salida = capfd.readouterr().out
    assert "ab" in salida and "cd" in salida
    assert salida.startswith("\x1b[?25l") and "\x1b[?25h" in salida


def test_cli_animar(tmp_path):
    destino = tmp_path / "a.svg"
    r = _cli("animar", str(LOGO), "-w", "30", "-t", "hola", "--svg", str(destino))
    assert r.returncode == 0, r.stderr
    assert destino.read_text(encoding="utf-8").count('<g class="f"') == 5
    # Sin terminal, la animación da una sola vuelta y termina.
    r = _cli("animar", str(LOGO), "-w", "20", "-t", "ab", "--intervalo", "0")
    # 3 fotogramas: se dibuja el primero y se sube 2 veces para redibujar.
    assert r.returncode == 0 and len(re.findall(r"\x1b\[\d+A", r.stdout)) == 2
    r = _cli(str(LOGO), "-w", "20", "-t", "ab", "--latido", "0.2")
    assert r.returncode == 2 and "--latido" in r.stderr
