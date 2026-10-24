# Fase 10 — el instalador: la app en cualquier Linux

La app tiene que dejar de ser "el repo con una ventana encima" y pasar a ser
un programa que se instala y se usa. Este es el camino que el lector pidió
primero, medido, y después el resto de los sistemas.

El trabajo de fondo ya está hecho: el motor va dentro del binario, los
modelos se abren con el diálogo del sistema (no con una carpeta fija) y los
recientes viven en el directorio de datos de la app. Falta lo que hace que
eso sea un instalable.

## Specs

### S1 — El bundle dice qué iconos usa y qué formatos se hacen

La configuración del bundle (`crates/genmsi-app/src-tauri/tauri.conf.json`)
tiene la lista de iconos **vacía** (`"icon": []`), y los iconos están en
`crates/genmsi-app/src-tauri/icons/` (los que genera Tauri: 32x32, 128x128,
128x128@2x, el `.icns` y el `.ico`).

**Lo que tiene que quedar**: la lista de iconos es la que hay, y los
formatos del paquete son los que se pueden hacer: **`.deb`** y **`.rpm`** se
generan en local sin red, y el **AppImage** descarga su propio runtime (hay
red, pero es lo único que depende de ella).

### S2 — La app no depende del repositorio en tiempo de ejecución

`app_root()` (`crates/genmsi-app/src-tauri/src/lib.rs`) arma la ruta del
repositorio con `env!("CARGO_MANIFEST_DIR")`: la ruta **tal como estaba al
compilar** queda horneada en el binario. Al arrancar, la app se mueve a esa
ruta; si no existe (que es lo que pasa en cualquier otra PC), avisa por la
consola y sigue.

Los modelos ya no se leen de esa carpeta (la app los abre con el diálogo), así
que el aviso es ruido, y la lista de modelos que el comando de la lista
devuelve tiene que ser una lista de modelos **de la app**, no de un repo que
puede no estar.

**Lo que tiene que quedar**: la app arranca en una máquina donde no está el
repositorio, sin avisos ni errores, y todo lo que necesita (los modelos que
el usuario elija, los recientes) está en el sistema de archivos del usuario.

### S3 — El paquete se genera, se instala y corre

El bundle se genera con `cargo tauri build` (la herramienta ya está
instalada: `tauri-cli 2.12.4`), se instala (`dpkg -i` para el `.deb`) y la
app instalada **abre una ventana y funciona**.

### S4 — Los datos del paquete

El `.deb` y el `.rpm` llevan nombre, versión, mantenedor, licencia y
descripción. Tauri los toma del `Cargo.toml` del crate de escritorio y de la
configuración del bundle: hay que revisarlos y dejarlos correctos, porque
son lo que ve el usuario cuando instala.

## Lo que se hizo y lo que se comprobó

### El bundle

`cargo tauri build` (el `tauri-cli 2.12.4` que ya estaba instalado) tarda
unos tres minutos y deja tres cosas en
`crates/genmsi-app/src-tauri/target/release/bundle/`:

| formato | para qué | cómo anda |
|---|---|---|
| `deb/GenMSI_0.1.0_amd64.deb` (3,9 MiB) | Debian, Ubuntu y derivados | **funciona**: se abre la ventana y la app habla con el motor |
| `rpm/GenMSI-0.1.0-1.x86_64.rpm` (3,9 MiB) | Fedora, openSUSE y derivados | el mismo programa con las mismas dependencias |
| `genmsi-0.1.0-linux-x86_64.tar.gz` (3,9 MiB) | cualquier Linux de escritorio, sin instalar | **funciona**: el programa, el lanzador, el icono y un `LEEME.txt` |

El `.deb` declara `libwebkit2gtk-4.1-0` y `libgtk-3-0` (las librerías del
sistema), y trae el lanzador (`GenMSI.desktop`, con el nombre, el `Exec`, el
icono y la categoría `Education`) y los iconos del tema `hicolor`.

**Comprobado**, no supuesto: el `.deb` se descomprimió, su programa se corrió
en un `Xvfb` y **abrió la ventana** (1440×900, la del tamaño de la
configuración), con la barra de herramientas, el «Abrir…», la semilla ancha y
el motor respondiendo (`session: una petición del frontend`, y los recientes
leídos del directorio de datos de la app). El portable hace lo mismo.

### El AppImage: no anda, y por qué

El primer bundle incluía también el `AppImage` (109 MiB, con el WebKit
empaquetado dentro) y **no funciona**: el WebKitGTK empaquetado busca su
proceso auxiliar en una ruta que depende de `$ORIGIN` y dentro del AppImage
queda relativa, así que el webview no arranca:

```
Unable to spawn a new child process: Falló al ejecutar el proceso hijo
«././/lib/webkit2gtk-4.1/WebKitNetworkProcess» (No existe el fichero o el
directorio)
```

Los auxiliares están dentro (`usr/lib/webkit2gtk-4.1/WebKitNetworkProcess` y
`WebKitWebProcess`), pero la ruta con la que se los busca sale de la
librería parcheada por `linuxdeploy` (con `RUNPATH $ORIGIN`) y no resuelve.

Es un límite de empaquetar WebKitGTK en un AppImage, no del programa: el
`.deb` y el portable usan el WebKit del sistema y funcionan. Como el AppImage
no servía, **se sacó de los formatos del bundle** (`"targets": ["deb",
"rpm"]`) para no dejar un artefacto roto, y se hizo el portable, que cubre lo
mismo (un Linux cualquiera, sin instalar) y sí anda.

### Lo que falta para los otros sistemas

- **macOS y Windows**: la configuración del bundle es la misma, pero el
  instalador de esos sistemas tiene que generarse **en esos sistemas**
  (Tauri no los genera desde aquí). Lo propio es una CI con una matriz.
- **Firmar los paquetes**: en Linux no hace falta para instalar, pero un
  `.deb` sin firma avisa al instalarlo, y las tiendas de aplicaciones lo
  piden.
- **La versión**: el primer instalable es el `0.1.0`; cuando la app esté
  lista para usar de verdad, la `0.2.0` con lo que falte.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1, S4 | La configuración del bundle: iconos, formatos, descripción y los datos del paquete | app | uno |
| T2 | S2 | La app sin el repositorio: la ruta del repo deja de ser necesaria en tiempo de ejecución | app | uno |
| T3 | S3 | El bundle se genera, se instala y la app instalada corre | — | — |

El orden: T1 y T2 juntos (son el mismo cambio, la configuración y el
arranque), y T3 es la prueba.

## Log

- **L1** — "no te preocupes era un archivo de test" — el `modelo de test` de la
  raíz del repositorio es material de trabajo, no del repo: no se commitea.
- **L2** — "ahora quiero que creemos la apllicacion para usarla en cualquier
  pc, por lo menos por ahora solo para linux te parece? o crees que hay que
  hacer algo mas?" — el instalable, primero en Linux.
- **L3** — "no te preocupes era un archivo de test" — el `modelo de test` de la
  raíz del repositorio es material de trabajo, no del repo: no se commitea.
  (`L1` y `L3` son la misma cosa, en el orden en que se dijeron.)
