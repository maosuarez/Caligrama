#!/usr/bin/env bash
# Recorrido por el CLI de caligrama. Ejecuta: bash examples/demo.sh
set -euo pipefail
cd "$(dirname "$(realpath "$0")")/.."
IMG=tests/data/python.jpeg

paso() { printf '\n\033[1m$ %s\033[0m\n' "$*"; "$@"; }

paso caligrama "$IMG" -w 40 -t "Hola mundo"
paso caligrama analizar "$IMG" -w 40 -t "Podrá nublarse el sol eternamente"
paso caligrama "$IMG" -w auto --sin-repetir -t "Podrá nublarse el sol eternamente; podrá secarse en un instante el mar."
paso caligrama "$IMG" -w 40 --espacios sin -t "Python"
paso caligrama "$IMG" -w 40 --invertir -t "fondo"
echo 'Un poema desde stdin' | { printf '\n\033[1m$ echo ... | caligrama %s -w 30\033[0m\n' "$IMG"; caligrama "$IMG" -w 30; }
