# Versionado e instaladores: la versión tiene una sola fuente y cada sistema recibe su instalador

La auditoría del 30/10 encontró que la versión vive en siete copias a mano y que
los instaladores tienen tres defectos vivos: el paquete de pacman nunca se generó,
el icono del paquete no aparece y el `pkgver` se desincroniza en la próxima
versión. Esta unidad cierra los siete huecos y blinda el versionado.

## Specs

### S1 — Una sola fuente de la versión, y un guardián que la vigila

Decisión del usuario: **"Todo, y con el versionado blindado"**.

Hoy la versión está escrita a mano en siete lugares y nada avisa cuando se
separan:

| Lugar | Campo |
|---|---|
| `Cargo.toml` | `[workspace.package] version` — **la fuente** |
| `crates/genmsi-app/engine/Cargo.toml` | `version` (escrita a propósito: el engine se compila desde el workspace propio del shell y no puede heredar) |
| `crates/genmsi-app/package.json` | `version` |
| `crates/genmsi-app/src-tauri/Cargo.toml` | `version` |
| `crates/genmsi-app/src-tauri/tauri.conf.json` | `version` — **se borra**: si falta, Tauri usa la del `Cargo.toml` del shell (documentado por Tauri) |
| `packaging/arch/PKGBUILD` | `pkgver=` — una sola línea literal, con el contrato de que el guardián la lee |
| `crates/genmsi-app/LEEME.txt` | la versión en el título y en los nombres de archivo |

**Lo que tiene que quedar**:

- `scripts/bump-version.sh <versión>` escribe las seis copias que siguen a la
  fuente y **falla** si deja alguna con una versión distinta. No cambia la
  fuente: la fuente la cambia el humano, a mano, en `Cargo.toml`.
- `crates/genmsi-app/tests/version.test.ts` lee las siete y falla si no
  coinciden. Corre con `bun run test`, que la CI ya corre en cada push.
- El `LEEME.txt` se genera desde la fuente: su título y los nombres de archivo
  que nombra llevan la versión.
- El `PKGBUILD` conserva **exactamente una** línea `pkgver=<versión>`: es el
  contrato que el guardián lee y que `build-package.sh` reescribe.

### S2 — La app dice qué versión es

Decisión del usuario: **"Todo, y con el versionado blindado"** → *mostrar la
versión en la app (Acerca de)*.

Hoy no hay forma de saber qué versión está instalada: `grep 0.3.0` en
`crates/genmsi-app/src` no da nada.

**Lo que tiene que quedar**:

- El shell expone un comando `app_version` que devuelve `env!("CARGO_PKG_VERSION")`
  del crate del shell (la versión que viaja en el bundle).
- El adaptador de la app gana `appVersion()`: el backend de Tauri lo pide al
  shell y el puente de desarrollo contesta con la de `package.json`.
- En la cabecera, la palabra **GenMSI** deja de ser texto: es un botón que abre
  un cuadro *Acerca de GenMSI* con la versión, los formatos que se publican y
  la licencia.
- El cuadro se abre con el teclado y se cierra con `Escape`.

### S3 — El paquete de pacman se genera y llega al release

Pendiente de la sesión anterior, textual del usuario: **"suma el instalador de
pacman por favor, ya que yo uso cachy os y quiero poder instalarlo"**.
Decisión del usuario: **"Primero relanzo la CI, después publicás vos"**.

El job de pacman falla desde que existe:

```
error: unzip is required to install bun
```

(el instalador de Bun necesita `unzip`, que no está en el contenedor de Arch).
Los dos últimos runs de Release fallaron por esto y el borrador del release
`v0.3.0` tiene cinco assets, ninguno de pacman.

**Lo que tiene que quedar**:

- El contenedor de pacman instala `unzip` y el job no vuelve a fallar.
- El `.pkg.tar.zst` se sube al release (el job `publicar-pacman` ya existe).

### S4 — El paquete de pacman está bien: icono y versión

**Lo que tiene que quedar**:

- El lanzador declara `Icon=GenMSI` (el nombre con el que Tauri instala el
  icono en el `.deb`) y el PKGBUILD instala el icono con ese nombre.
  `scripts/install-linux.sh` instala el icono con el mismo nombre.
- `build-package.sh` reescribe **también** `pkgver=` (además de `source=` y
  `sha256sums=`), de modo que el paquete se arma con la versión del código que
  se está armando y no con la del `PKGBUILD` del repositorio.

### S5 — El PKGBUILD y el `.SRCINFO` quedan listos para la AUR

**Lo que tiene que quedar**:

- El job de pacman publica `PKGBUILD` y `.SRCINFO` como assets del release,
  con la suma real del tarball de la fuente.
- El `PKGBUILD` publicado lleva `pkgver` literal y `sha256sums` con la suma
  real (la AUR no acepta `SKIP`; el `PKGBUILD` del repositorio conserva el
  `SKIP` con un comentario que dice quién lo rellena).
- **Alcance declarado**: el repositorio es **privado** y la AUR no puede bajar
  un tarball de un repositorio privado. El paquete queda listo; publicarlo en
  la AUR exige que la fuente sea pública, que es una decisión del usuario. La
  instalación que sí funciona hoy con el repositorio privado es
  `pacman -U genmsi-<versión>-1-x86_64.pkg.tar.zst` sobre el asset del release.

### S6 — La CI compila y prueba el shell de escritorio

**Lo que tiene que quedar**: `checks.yml` compila y prueba
`crates/genmsi-app/src-tauri`, que hoy el workspace excluye (`Cargo.toml`,
`exclude`) y que por lo tanto **nunca se compila en la CI**. Los avisos de
`gtk3` y `webkit2gtk-4.1` que el shell necesita quedan instalados.

### S7 — El release se puede verificar

**Lo que tiene que queda**r:

- Un `SHA256SUMS` con la suma de cada instalador del release.
- El cuerpo del release dice qué hay en cada archivo y cómo se instala en cada
  sistema.
- El `workflow_dispatch` no lleva la versión escrita a mano: la toma de la
  etiqueta o la deja vacía.

### S8 — macOS: un instalador para los dos Macs

El `.dmg` de hoy es **aarch64** (`macos-latest` es Apple Silicon): los Mac
Intel no tienen instalador.

**Lo que tiene que quedar**: un `.dmg` universal (Intel + Apple Silicon).

### S9 — Linux arm64

**Lo que tiene que quedar**: `.deb` y `.rpm` de aarch64, construidos en un
runner arm64. El runner arm64 de un repositorio privado se factura aparte: el
costo por release es de céntimos.

### S10 — Flatpak

**Lo que tiene que quedar**: el manifiesto de Flatpak y el paquete construidos
en la CI, con el `.flatpakref` publicado. El Flatpak usa el WebKitGTK del
sistema (a diferencia del AppImage, que quedó fuera por un bug conocido de
Tauri), así que funciona en cualquier distro.

### S11 — El contrato entre las tres unidades

Las tres unidades se arman en paralelo y no se ven entre sí. Los contratos
que las unen:

1. El `PKGBUILD` tiene **exactamente una** línea `pkgver=<versión>`.
2. La versión de la fuente es `Cargo.toml` → `[workspace.package] version`.
3. Los formatos que se publican son, y son sólo estos:
   `.deb` (x86_64 y aarch64), `.rpm` (x86_64 y aarch64),
   `.pkg.tar.zst` (x86_64), `.dmg` (universal), `-setup.exe` y `.msi` (x64),
   `.flatpak`, más el `.tar.gz` portable y el `SHA256SUMS`.
4. El `LEEME.txt` lo escribe la unidad del versionado y nombra esos formatos.
   La unidad del empaquetado **no** lo toca.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1 | La fuente única: `scripts/bump-version.sh`, el guardián `version.test.ts`, la versión fuera de `tauri.conf.json`, el `LEEME.txt` derivado | app | `949aac9` |
| T2 | S2 | El *Acerca de*: el comando `app_version`, el adaptador, el cuadro de la cabecera | app | `cb74a82` |
| T3 | S3, S6 | La CI: `unzip` en el contenedor de pacman y el shell de escritorio en las pruebas | ci | `793bdd7` |
| T4 | S4 | El paquete de pacman: el nombre del icono y el `pkgver` del script | packaging | `e8c53de` |
| T5 | S5 | El `PKGBUILD` y el `.SRCINFO` publicados como assets del release | packaging | `e8c53de` |
| T6 | S7 | El `SHA256SUMS`, el cuerpo del release y el `workflow_dispatch` sin versión escrita | ci | `793bdd7` |
| T7 | S8 | El `.dmg` universal | ci | `793bdd7` |
| T8 | S9 | Los `.deb` y `.rpm` de aarch64 | ci | `793bdd7` |
| T9 | S10 | El Flatpak: el manifiesto y el job de la CI | app + ci | `e8c53de`, `793bdd7` |

Los tres commits de corrección que salieron de la verificación van aparte:
`d26846a` (la prueba de la huella del shell), `7da2401` (el script de la
versión y el guardián que se prueba a sí mismo) y `f79e36b` (el nombre del
`.SRCINFO`).

El orden: T1 y T2 son la unidad del versionado; T3 a T9 son la del
empaquetado. T9 toca las dos (el manifiesto es de la app, el job es de la CI).

## Log

- **L1** — "Hola! hoy quiero que revisemos si esta creada correctamente el
  versionado de mi aplicacion y la creacion de los insataladores para los
  diferentes so. Quiero que analices y busques si hay mejores formas de
  hacerlo o si estann bien creados. ademas de que teniamos cosas pendientes que
  no me acuerdo que eran" — la revisión.
- **L2** — "Todo, y con el versionado blindado" — el alcance que eligió el
  usuario sobre las cuatro opciones que le ofrecí.
- **L3** — "Primero relanzo la CI, después publicás vos" — la publicación del
  borrador del release es del usuario; la CI la relanzo yo.
- **L4** — La auditoría (30/10) encontró, con evidencia: el job de pacman
  falla por `unzip` (`release.yml:115`) y el `.pkg.tar.zst` nunca existió; el
  lanzador declara `Icon=genmsi-app-desktop` (`GenMSI.desktop:6`) y el PKGBUILD
  instala el icono como `GenMSI.png` (`PKGBUILD:46`); `build-package.sh:52`
  reescribe `source=` y `sha256sums=` pero no `pkgver=`; `PKGBUILD:28` tiene
  `sha256sums=('SKIP')`; el `.dmg` es aarch64; `checks.yml:32` excluye el shell
  (`Cargo.toml:13`); el `tauri.conf.json` lleva una copia de la versión que
  Tauri no necesita; la app no muestra su versión en ninguna parte.
- **L5** — El repositorio es **privado** (`gh repo view`: `isPrivate: true`).
  Dos consecuencias: la AUR no puede bajar la fuente (S5 lo dice y el paquete
  queda listo, pero publicarlo en la AUR exige fuente pública: decisión del
  usuario), y el runner arm64 se factura (S9: céntimos por release).
- **L6** — Descartadas dos alternativas, con la razón: `cargo-dist` no sirve
  acá (sus instaladores son shell/powershell/npm/homebrew/msi/macpkg: no cubren
  una app Tauri con webview ni el formato de pacman), y el AppImage se queda
  fuera (el error del `WebKitNetworkProcess` es un bug conocido de Tauri —
  tauri#5292, #2805, #15665 —, no del proyecto).
- **L7** — La entrega: rama `feature/versionado-instaladores`, tres unidades en
  paralelo (versionado, empaquetado, CI), la verificación de las de alto riesgo
  aparte, y al final la CI relanzada sobre `main` para que el borrador del
  release `v0.3.0` quede con los seis formatos. La publicación es del usuario
  (L3).
- **L8** — La entrega que eligió el usuario: **"Los slices a main, uno por
  uno"**. Cada unidad de trabajo llega a `main` por separado y la CI la
  verifica en cada una (la cadena verificada contra `main`). Los commits:
  `949aac9` (T1), `cb74a82` (T2), `e8c53de` (T4, T5, el manifiesto de T9) y
  `793bdd7` (T3, T6, T7, T8, el job de T9). Los cuatro llegaron a `main` uno
  después del otro, con la CI en verde en cada uno.
- **L9** — El worker del empaquetado tradujo a inglés los tres archivos de
  empaquetado que estaban en español (el `PKGBUILD`, el script y el instalador
  de la app), y con ellos los mensajes que ve el usuario. Eso no era parte del
  encargo y cambiaba el idioma de la salida sin que nadie lo pidiera: se
  revirtió y se conservaron sólo los cambios funcionales (el nombre del icono
  y el `pkgver`). Los archivos de empaquetado siguen en español, como estaban.
- **L10** — El script de la versión se corrigió antes de aceptarlo: escribía
  las seis copias y **después** verificaba la fuente, así que una persona que
  olvidara editar el `Cargo.toml` dejaba el árbol con dos versiones. Ahora la
  fuente se verifica **antes** de escribir, y el guardián comprueba que no se
  escribió nada.
- **L11** — El nombre del icono: el `.deb` y el `.rpm` de Tauri instalan el
  icono como `genmsi-app-desktop.png` y su lanzador lo pide con ese nombre
  (comprobado abriendo el `.deb` construido); los tres instaladores que usan
  el lanzador del repositorio (el de pacman, el de usuario y el Flatpak) usan
  `GenMSI`. Cada instalador es coherente consigo mismo.
- **L12** — Lo verificado y lo que no: en local, 610 pruebas del motor (86
  suites), 323 de la app, 11 del shell, el typecheck, el build del frontend,
  los dos workflows como YAML válido con todos sus `needs` resueltos y todos
  sus `run:` compados por `bash -n`, el `PKGBUILD` con `makepkg
  --printsrcinfo`, y el script de la versión sobre copias en un directorio
  temporal. **No** se verificó en local: el `.dmg` universal, el runner
  arm64, el build del Flatpak (no hay `flatpak-builder` en la máquina) y el
  cuerpo del release contra un release real — eso lo prueba la corrida de la
  CI. La descarga de los assets del borrador con `gh release download` se
  comprobó contra el borrador real de `v0.3.0`: funciona con borradores.
- **L13** — **El job nuevo del shell encontró un defecto real**: la CI
  nunca había compilado el shell (el workspace lo excluye), y su prueba de
  la huella de un archivo fallaba en el runner: escribía el archivo dos
  veces y exigía que las fechas fueran distintas, cuando lo que manda es la
  **granularidad del reloj del filesystem**, no la unidad del valor. La
  prueba ahora espera al reloj en vez de suponerlo (`d26846a`).
- **L14** — El verificador independiente (`gentle-ai-verify`) no encontró
  **ningún bloqueo**, y sus avisos se atendieron así: (1) el AppImage: es un
  **falso positivo** — el config por plataforma reduce `targets: "all"` a
  `[deb, rpm]` en Linux, y tanto el bundle local como el release de la CI lo
  demuestran (no hay AppImage); además ahora hay un test que lo vigila.
  (2) El script aceptaba `0.4.0-rc1` y fallaba con el árbol escrito: ahora
  habla sólo `X.Y.Z` y falla antes de escribir (`7da2401`). (3) El
  `.SRCINFO` se publicaba con el nombre equivocado: ahora se llama como la
  AUR lo llama (`f79e36b`). (4) Los cambios cosméticos de
  `scripts/install-linux.sh` se revirtieron: su diff contra la base es
  sólo el nombre del icono. (5) El `.deb` de Tauri llama al icono
  `genmsi-app-desktop.png` y el nuestro `GenMSI.png`: cada instalador es
  coherente consigo mismo (L11). (6) Sin test del cuadro *Acerca de*: no hay
  DOM en el runner, declarado como excepción de aplicabilidad.
- **L15** — El guardián dejó de ser un test que nadie había visto fallar:
  ahora hay dos pruebas que lo hacen fallar a propósito (una copia con la
  versión desalineada) y una que comprueba que el script no escribe nada
  cuando la fuente no coincide (`7da2401`). La app pasó de 311 a 327
  pruebas.
