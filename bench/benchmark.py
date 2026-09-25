"""Benchmark: caligrama contra otras formas de dibujar una imagen con texto.

    pip install -r bench/requirements.txt    # los competidores de Python
    cargo install viu; sudo apt install catimg   # opcionales: los que falten se omiten
    python bench/benchmark.py                # ancho 70, imágenes de assets/ y tests/data/
    python bench/benchmark.py -w 60 foto.png # otra imagen
    python bench/benchmark.py --ver          # además imprime cada dibujo

Mide lo mismo para todos y escribe bench/resultados/informe.html (y visores.html) con los dibujos
lado a lado (con su color) y una vitrina de todas las opciones de caligrama.

Métricas (ninguna depende de caligrama):
- ms: mediana de varias ejecuciones (sin contar el import; en los programas
  externos, el proceso entero).
- import ms: lo que tarda en importarse la solución en un proceso nuevo.
- paquetes: distribuciones que se instalan (dependencias transitivas incluidas).
- legibles: % de palabras del dibujo que son palabras del poema, enteras.
- en orden: % de letras que siguen el poema en su orden.
- poema: % del poema que se lee completo y en orden desde el principio.
- colores: colores distintos en el dibujo.
"""

import argparse
import html
import re
import shutil
import statistics
import subprocess
import sys
import time
from importlib import metadata
from pathlib import Path

from competidores import todos

AQUI = Path(__file__).resolve().parent
RAIZ = AQUI.parent
SALIDA = AQUI / "resultados"
IMAGENES = ["assets/arbol.png", "tests/data/python.jpeg"]

POEMA = """
Quiero crecer contigo,
como crece este árbol:
despacio,
con raíces fuertes,
con ramas que buscan el cielo
y hojas que guardan
cada momento que vivimos.
"""

# --- ANSI -> celdas ------------------------------------------------------------

BASICOS = [
    (0, 0, 0), (205, 49, 49), (13, 188, 121), (229, 229, 16),
    (36, 114, 200), (188, 63, 188), (17, 168, 205), (229, 229, 229),
]
BRILLANTES = [
    (102, 102, 102), (241, 76, 76), (35, 209, 139), (245, 245, 67),
    (59, 142, 234), (214, 112, 214), (41, 184, 219), (255, 255, 255),
]
SGR = re.compile(r"\x1b\[([\d;]*)m")


def _color_256(n):
    if n < 8:
        return BASICOS[n]
    if n < 16:
        return BRILLANTES[n - 8]
    if n < 232:
        n -= 16
        return tuple(0 if v == 0 else 55 + v * 40 for v in (n // 36, n // 6 % 6, n % 6))
    g = 8 + (n - 232) * 10
    return (g, g, g)


def celdas(texto):
    """Líneas de (carácter, color de letra, color de fondo) leyendo los códigos
    SGR. El fondo hace falta para los visores de medio bloque (viu, catimg)."""
    lineas, fg, bg = [], None, None
    for linea in texto.replace("\r", "").rstrip("\n").split("\n"):
        # Secuencias que no son color (guardar cursor, ocultarlo...) no son celdas.
        linea = re.sub(r"\x1b\[[\d;?]*[A-Za-ln-z]", "", linea)
        fila, pos = [], 0
        for m in SGR.finditer(linea):
            fila += [(c, fg, bg) for c in linea[pos : m.start()]]
            pos = m.end()
            p = [int(x) if x else 0 for x in m.group(1).split(";")]
            i = 0
            while i < len(p):
                v = p[i]
                if v == 0:
                    fg = bg = None
                elif v == 39:
                    fg = None
                elif v == 49:
                    bg = None
                elif v in (38, 48) and p[i + 1 : i + 2] == [2]:
                    c, i = tuple(p[i + 2 : i + 5]), i + 4
                    fg, bg = (c, bg) if v == 38 else (fg, c)
                elif v in (38, 48) and p[i + 1 : i + 2] == [5]:
                    c, i = _color_256(p[i + 2]), i + 2
                    fg, bg = (c, bg) if v == 38 else (fg, c)
                elif 30 <= v <= 37:
                    fg = BASICOS[v - 30]
                elif 90 <= v <= 97:
                    fg = BRILLANTES[v - 90]
                elif 40 <= v <= 47:
                    bg = BASICOS[v - 40]
                elif 100 <= v <= 107:
                    bg = BRILLANTES[v - 100]
                i += 1
        fila += [(c, fg, bg) for c in linea[pos:]]
        lineas.append(fila)
    return lineas


def plano(texto):
    return "\n".join("".join(c for c, _, _ in f).rstrip() for f in celdas(texto))


# --- Métricas ------------------------------------------------------------------


def metricas(dibujo, poema):
    txt = plano(dibujo)
    vocab = set(re.findall(r"\w+", poema.lower()))
    fichas = re.findall(r"\w+", txt.lower())
    legibles = sum(f in vocab for f in fichas) / len(fichas) if fichas else 0.0

    ref = re.sub(r"\s", "", poema)
    letras = re.sub(r"\s", "", txt)
    en_orden = (
        sum(c == ref[i % len(ref)] for i, c in enumerate(letras)) / len(letras) if letras else 0.0
    )
    prefijo = next((i for i, (a, b) in enumerate(zip(letras, ref)) if a != b), min(len(letras), len(ref)))

    colores = {fg for f in celdas(dibujo) for ch, fg, _ in f if fg and not ch.isspace()}
    colores |= {bg for f in celdas(dibujo) for _, _, bg in f if bg}
    return {
        "celdas": len(letras),
        "legibles": legibles,
        "en_orden": en_orden,
        # Coincidir por azar en 1-2 letras (la "Q" de ascii_magic) no es leer el poema.
        "poema": prefijo / len(ref) if prefijo >= 5 else 0.0,
        "colores": len(colores),
    }


def cronometrar(fn, *args, veces=5):
    fn(*args)  # calentar (y cargar el módulo)
    tiempos = []
    for _ in range(veces):
        t = time.perf_counter()
        fn(*args)
        tiempos.append(time.perf_counter() - t)
    return statistics.median(tiempos) * 1000


def tiempo_import(modulos, veces=3):
    codigo = (
        "import time,contextlib,os\n"
        "t=time.perf_counter()\n"
        "with open(os.devnull,'w') as n, contextlib.redirect_stderr(n), contextlib.redirect_stdout(n):\n"
        + "".join(f"    import {m}\n" for m in modulos)
        + "print(time.perf_counter()-t)"
    )
    ms = []
    for _ in range(veces):
        r = subprocess.run([sys.executable, "-c", codigo], capture_output=True, text=True, cwd="/")
        if r.returncode:
            return None
        ms.append(float(r.stdout.split()[-1]) * 1000)
    return statistics.median(ms)


def paquetes(nombres):
    """Distribuciones instaladas por `nombres`, dependencias transitivas incluidas."""
    vistos, pila = set(), list(nombres)
    while pila:
        n = re.split(r"[\s;<>=!~\[(]", pila.pop(), maxsplit=1)[0].lower().replace("_", "-")
        if n in vistos:
            continue
        try:
            dist = metadata.distribution(n)
        except metadata.PackageNotFoundError:
            continue
        vistos.add(n)
        pila += [r for r in (dist.requires or []) if "extra ==" not in r]
    return len(vistos)


# --- HTML ------------------------------------------------------------------------


def a_html(dibujo):
    partes = []
    for fila in celdas(dibujo):
        linea, actual, tramo = [], object(), ""
        for ch, fg, bg in fila + [(None, None, None)]:
            if (fg, bg) != actual or ch is None:
                if tramo:
                    t = html.escape(tramo)
                    estilo = (f"color:rgb{actual[0]};" if actual[0] else "") + (
                        f"background:rgb{actual[1]}" if actual[1] else ""
                    )
                    linea.append(f'<span style="{estilo}">{t}</span>' if estilo else t)
                tramo, actual = "", (fg, bg)
            if ch is not None:
                tramo += ch
        partes.append("".join(linea))
    return "\n".join(partes)


CSS = """
:root{color-scheme:dark;--fondo:#101014;--tarjeta:#18181f;--borde:#2a2a35;--texto:#e8e6ef;--suave:#9a97a8;--acento:#ff2d55}
*{box-sizing:border-box}body{margin:0;background:var(--fondo);color:var(--texto);font:15px/1.5 system-ui,sans-serif}
main{max-width:1500px;margin:auto;padding:24px 16px 64px}h1{margin:0 0 4px}h2{margin:48px 0 12px;border-bottom:1px solid var(--borde);padding-bottom:6px}
p.sub{color:var(--suave);margin:0 0 16px}
table{border-collapse:collapse;width:100%;margin:8px 0 20px;font-variant-numeric:tabular-nums}
th,td{padding:6px 10px;border-bottom:1px solid var(--borde);text-align:right;white-space:nowrap}th:first-child,td:first-child{text-align:left}
th{color:var(--suave);font-weight:600}tr.cal td:first-child{color:var(--acento);font-weight:600}
.tabla{overflow-x:auto}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(420px,1fr));gap:14px}
.card{background:var(--tarjeta);border:1px solid var(--borde);border-radius:10px;padding:12px;min-width:0}
.card h3{margin:0 0 2px;font-size:15px}.card small{color:var(--suave);display:block;margin-bottom:8px}
pre{margin:0;overflow-x:auto;font:bold 8px/1.2 'DejaVu Sans Mono',Menlo,Consolas,monospace;color:#d6d3e0}
pre.bloques{line-height:1}
code{background:#23232d;padding:1px 5px;border-radius:4px}
.svg{background:#14111a;border-radius:10px;padding:8px;overflow:auto}.svg img{max-width:100%;height:auto;display:block}
"""


def pct(x):
    return f"{x * 100:.0f} %"


def tabla_html(filas):
    cab = ["Solución", "ms", "import ms", "paquetes", "celdas", "legibles", "en orden", "poema", "colores"]
    out = ["<div class='tabla'><table><tr>" + "".join(f"<th>{c}</th>" for c in cab) + "</tr>"]
    for r in filas:
        m = r["m"]
        imp = "—" if r["import"] is None else f"{r['import']:.0f}"
        clase = " class='cal'" if r["nombre"].startswith("caligrama") else ""
        out.append(
            f"<tr{clase}><td>{html.escape(r['nombre'])}</td><td>{r['ms']:.1f}</td><td>{imp}</td>"
            f"<td>{r['paquetes']}</td><td>{m['celdas']}</td><td>{pct(m['legibles'])}</td>"
            f"<td>{pct(m['en_orden'])}</td><td>{pct(m['poema'])}</td><td>{m['colores']}</td></tr>"
        )
    return "".join(out) + "</table></div>"


def tarjeta(titulo, nota, dibujo, clase=""):
    return (
        f"<div class='card'><h3>{html.escape(titulo)}</h3><small>{nota}</small>"
        f"<pre class='{clase}'>{a_html(dibujo)}</pre></div>"
    )


def vitrina(ancho):
    """Todas las opciones de caligrama, una tarjeta por opción."""
    import caligrama

    arbol, logo = RAIZ / "assets" / "arbol.png", RAIZ / "tests" / "data" / "python.jpeg"
    casos = [
        ("por defecto", "<code>dibujar(img, poema, ancho=%d)</code>" % ancho, arbol, {"ancho": ancho}),
        ("ancho=\"auto\"", "El menor ancho donde el poema cabe una sola vez, entero.", arbol,
         {"ancho": "auto", "repetir": False}),
        ("espacios=\"sin\"", "Sin espacios: figura más sólida, se lee peor.", arbol,
         {"ancho": ancho, "espacios": "sin"}),
        ("espacios=\"todos\"", "Respeta cada espacio y salto del poema.", arbol,
         {"ancho": ancho, "espacios": "todos"}),
        ("color=True", "Cada letra con el color de la imagen en su celda.", arbol,
         {"ancho": ancho, "color": True}),
        ("repetir=False", "El poema una sola vez; el resto de la figura queda vacía.", arbol,
         {"ancho": ancho, "repetir": False}),
        ("desfase=40", "Empieza el poema 40 letras más adelante.", arbol, {"ancho": ancho, "desfase": 40}),
        ("huecos=False (defecto)", "Rellena lo encerrado por la figura.", logo, {"ancho": ancho, "color": True}),
        ("huecos=True", "Conserva los huecos del color del fondo (el ojo del logo).", logo,
         {"ancho": ancho, "huecos": True, "color": True}),
        ("umbral=120", "Umbral manual: solo lo muy distinto al fondo es figura.", logo,
         {"ancho": ancho, "umbral": 120, "color": True}),
        ("invertir=True", "Escribe alrededor de la figura.", arbol, {"ancho": ancho, "invertir": True}),
        ("suavizar=6", "Cierre morfológico: une trazos finos antes de muestrear.", logo,
         {"ancho": ancho, "suavizar": 6, "color": True}),
        ("aspecto=1.0", "Para fuentes cuadradas (la figura sale el doble de alta aquí).", arbol,
         {"ancho": 40, "aspecto": 1.0}),
    ]
    tarjetas = []
    for titulo, nota, img, op in casos:
        try:
            dibujo = caligrama.dibujar(img, POEMA, **op)
        except ValueError as e:
            dibujo = f"(error: {e})"
        tarjetas.append(tarjeta(f"{titulo} · {img.name}", nota, dibujo))

    inf = caligrama.analizar(arbol, POEMA, ancho=ancho)
    resumen = (
        f"{inf['letras']} letras (~{inf['palabras_aprox']} palabras) en {inf['filas']} filas; "
        f"el poema tiene {inf['texto_letras']} y cabe entero desde <code>ancho={inf['ancho_recomendado']}</code>."
    )
    tarjetas.append(tarjeta("analizar(): plantilla", resumen, inf["plantilla"]))

    # Con color cada letra es un <tspan>: se limitan los fotogramas para que pese poco.
    fotos = caligrama.animar(arbol, POEMA, ancho=ancho, fotogramas=24, paso=2, latido=0.15, color=True)
    svg = caligrama.a_svg(fotos, intervalo=0.08, fondo="#14111a", tamano=10)
    (SALIDA / "arbol_animado.svg").write_text(svg, encoding="utf-8")
    anim = (
        f"<div class='card'><h3>animar() + a_svg()</h3><small>fotogramas=24, paso=2, latido=0.15, color=True · "
        f"{len(svg) // 1024} KB</small><div class='svg'><img src='arbol_animado.svg' alt='árbol animado'></div></div>"
    )
    return "<div class='grid'>" + "".join(tarjetas) + anim + "</div>"


# --- caligrama contra visores de imágenes (viu, catimg) -------------------------

VISORES = ["viu", "catimg"]
CALIGRAMAS = ["caligrama color", "caligrama sin espacios"]

CARACTERISTICAS = [
    # (qué, viu, catimg, caligrama)
    ("Color de 24 bits", "sí", "sí", "sí (<code>color=True</code>)"),
    ("Resolución por celda", "2 píxeles (▄ con letra y fondo)", "2 píxeles (▀ con letra y fondo)", "1 letra"),
    ("Escribe un mensaje o poema", "no", "no", "sí"),
    ("Separa la figura del fondo", "no: pinta el fondo también", "no: pinta el fondo también", "sí: el fondo queda vacío"),
    ("Sin color (copiar y pegar) conserva la forma", "no: un rectángulo de ▄", "no: un rectángulo de ▀", "sí: la forma son las letras"),
    ("Protocolos gráficos (Kitty, iTerm, Sixel)", "sí", "no", "no (no los necesita)"),
    ("Animación", "GIF", "GIF (<code>-l</code>)", "texto que fluye y latido; exporta SVG"),
    ("Se usa desde Python", "no (proceso aparte)", "no (proceso aparte)", "sí, devuelve un <code>str</code>"),
    ("Instalación", "<code>cargo install viu</code>", "<code>apt install catimg</code>", "<code>pip install caligrama</code>"),
]


def informe_visores(por_imagen, ancho):
    """resultados/visores.html: viu y catimg frente a caligrama."""
    nombres = VISORES + CALIGRAMAS
    if not any(c.nombre in VISORES for fila in por_imagen.values() for c, _, _ in fila):
        return False
    secciones = []
    for imagen, fila in por_imagen.items():
        elegidos = sorted(
            [x for x in fila if x[0].nombre in nombres], key=lambda x: nombres.index(x[0].nombre)
        )
        cab = ["Solución", "ms", "filas × columnas", "tamaño de la salida", "colores", "palabras legibles", "celdas pintadas"]
        tabla = ["<div class='tabla'><table><tr>" + "".join(f"<th>{h}</th>" for h in cab) + "</tr>"]
        color, plano_ = [], []
        for c, r, dibujo in elegidos:
            rejilla = celdas(dibujo)
            m = r["m"]
            clase = " class='cal'" if c.nombre.startswith("caligrama") else ""
            tabla.append(
                f"<tr{clase}><td>{html.escape(c.nombre)}</td><td>{r['ms']:.1f}</td>"
                f"<td>{len(rejilla)} × {max(map(len, rejilla))}</td>"
                f"<td>{len(dibujo.encode()) / 1024:.1f} KB</td><td>{m['colores']}</td>"
                f"<td>{pct(m['legibles'])}</td><td>{m['celdas']}</td></tr>"
            )
            bloques = "bloques" if c.familia == "visor" else ""
            color.append(tarjeta(c.nombre, html.escape(c.nota), dibujo, bloques))
            plano_.append(tarjeta(c.nombre, "Lo que queda al pegarlo donde no hay color.", plano(dibujo), bloques))
        tabla.append("</table></div>")
        secciones.append(
            f"<h2>{html.escape(imagen)}</h2>{''.join(tabla)}"
            f"<h3 class='sec'>Con color</h3><div class='grid g4'>{''.join(color)}</div>"
            f"<h3 class='sec'>Sin color (texto plano)</h3><div class='grid g4'>{''.join(plano_)}</div>"
        )

    filas_car = "".join(
        f"<tr><td>{q}</td><td>{v}</td><td>{ci}</td><td>{ca}</td></tr>" for q, v, ci, ca in CARACTERISTICAS
    )
    import caligrama

    pagina = (
        "<!doctype html><html lang='es'><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'>"
        f"<title>caligrama vs visores</title><style>{CSS}"
        ".g4{grid-template-columns:repeat(auto-fill,minmax(320px,1fr))}h3.sec{color:var(--suave);font-size:14px;margin:18px 0 8px}"
        ".car td{white-space:normal;text-align:left}.car td:first-child{color:var(--suave)}"
        ".veredicto{background:var(--tarjeta);border:1px solid var(--borde);border-left:3px solid var(--acento);border-radius:8px;padding:12px 16px;margin:16px 0}"
        "</style><main>"
        f"<h1>caligrama {caligrama.__version__} contra viu y catimg</h1>"
        f"<p class='sub'>Mismas imágenes, ancho {ancho} columnas. viu y catimg son visores: reproducen la foto con "
        "medios bloques de color. caligrama dibuja la silueta con las letras de un mensaje y le pone el color de la imagen. "
        "El tiempo de viu y catimg incluye arrancar el proceso, que es como se usan.</p>"
        "<div class='veredicto'><b>Para ver una foto en la terminal</b>, viu y catimg son más fieles: dos píxeles de color por celda, "
        "fondo incluido. <b>Para mandar un mensaje con forma</b>, solo caligrama sirve: es lo único que se lee, lo único que "
        "conserva la forma como texto plano (WhatsApp, un README, un commit). En velocidad los tres están en pocos "
        "milisegundos: con color, caligrama gana en unas imágenes y viu o catimg en otras (ver tablas).</div>"
        f"<h2>Qué hace cada uno</h2><div class='tabla'><table class='car'><tr><th></th><th>viu</th><th>catimg</th>"
        f"<th>caligrama</th></tr>{filas_car}</table></div>"
        + "".join(secciones)
        + "</main></html>"
    )
    (SALIDA / "visores.html").write_text(pagina, encoding="utf-8")
    return True


# --- Principal ---------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("imagenes", nargs="*", default=IMAGENES)
    ap.add_argument("-w", "--ancho", type=int, default=70)
    ap.add_argument("-n", "--veces", type=int, default=5, help="ejecuciones por medición")
    ap.add_argument("--ver", action="store_true", help="imprimir cada dibujo en la terminal")
    args = ap.parse_args()
    SALIDA.mkdir(exist_ok=True)

    from rich.console import Console
    from rich.table import Table

    consola = Console()
    competidores = []
    for c in todos():
        if c.binario:
            if shutil.which(c.binario) is None:
                consola.print(f"[yellow]· {c.nombre}: no instalado ({c.instalar}), se omite[/]")
            else:
                competidores.append((c, None, "binario"))
            continue
        imp = tiempo_import(c.modulos)
        if imp is None:
            consola.print(f"[yellow]· {c.nombre}: no instalado (pip install {' '.join(c.paquetes)}), se omite[/]")
            continue
        competidores.append((c, imp, paquetes(c.paquetes)))

    secciones, por_imagen = [], {}
    for nombre in args.imagenes:
        img = Path(nombre) if Path(nombre).exists() else RAIZ / nombre
        filas, tarjetas = [], []
        for c, imp, npaq in competidores:
            dibujo = c.fn(img, POEMA, args.ancho)
            ms = cronometrar(c.fn, img, POEMA, args.ancho, veces=args.veces)
            filas.append({"nombre": c.nombre, "ms": ms, "import": imp, "paquetes": npaq, "m": metricas(dibujo, POEMA)})
            por_imagen.setdefault(img.name, []).append((c, filas[-1], dibujo))
            # Los medios bloques deben tocarse entre filas para verse como píxeles.
            tarjetas.append(tarjeta(c.nombre, html.escape(c.nota), dibujo, "bloques" if c.familia == "visor" else ""))
            slug = re.sub(r"\W+", "_", c.nombre).strip("_")
            (SALIDA / f"{img.stem}_{slug}.txt").write_text(dibujo, encoding="utf-8")
            if args.ver:
                consola.rule(f"{c.nombre} · {img.name}")
                print(dibujo)

        t = Table(title=f"{img.name} · ancho {args.ancho}")
        for col in ["Solución", "ms", "import ms", "paquetes", "celdas", "legibles", "en orden", "poema", "colores"]:
            t.add_column(col, justify="left" if col == "Solución" else "right")
        for r in filas:
            m = r["m"]
            t.add_row(
                r["nombre"], f"{r['ms']:.1f}", "—" if r["import"] is None else f"{r['import']:.0f}", str(r["paquetes"]), str(m["celdas"]),
                pct(m["legibles"]), pct(m["en_orden"]), pct(m["poema"]), str(m["colores"]),
                style="bold magenta" if r["nombre"].startswith("caligrama") else None,
            )
        consola.print(t)
        secciones.append(
            f"<h2>{html.escape(img.name)}</h2>{tabla_html(filas)}<div class='grid'>{''.join(tarjetas)}</div>"
        )

    import caligrama

    pagina = (
        "<!doctype html><html lang='es'><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'>"
        f"<title>Benchmark caligrama</title><style>{CSS}</style><main>"
        f"<h1>caligrama {caligrama.__version__} contra otras soluciones</h1>"
        f"<p class='sub'>Mismo poema, mismas imágenes, ancho {args.ancho}. Python {sys.version.split()[0]}. "
        "<b>legibles</b>: palabras del dibujo que son palabras enteras del poema · <b>en orden</b>: letras que siguen el poema · "
        "<b>poema</b>: parte del poema que se lee entera y en orden · <b>paquetes</b>: lo que se instala, dependencias incluidas.</p>"
        + "".join(secciones)
        + "<p class='sub'>Fuera de la comparación: <b>lsix</b> (Sixel) y <b>fim</b> (framebuffer) dibujan píxeles, "
        "no texto; su salida no es un <code>str</code> que se pueda medir, copiar o pegar. viu y catimg sí escriben texto "
        "(bloques ▀ con color), pero reproducen la foto, no un mensaje.</p>"
        + f"<h2>Todas las opciones de caligrama</h2>{vitrina(args.ancho)}</main></html>"
    )
    (SALIDA / "informe.html").write_text(pagina, encoding="utf-8")
    consola.print(f"\n[green]Informe:[/] {SALIDA / 'informe.html'}  ·  dibujos en {SALIDA}/")
    if informe_visores(por_imagen, args.ancho):
        consola.print(f"[green]Visores:[/] {SALIDA / 'visores.html'}")


if __name__ == "__main__":
    main()
