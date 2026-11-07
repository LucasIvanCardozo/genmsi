# Limpieza de atribución: Dependabot fuera, co-autores fuera

Feature doc de ODD. La historia publicada del repositorio se reescribe: se borran los
pies de atribución (co-autores de Claude y la firma de Dependabot) y se elimina
Dependabot. El objetivo es que ningún commit del proyecto quede firmado por alguien
que no es su dueño.

Los subagentes leen este documento entero, hasta `## Log`, y dicen qué `S#` cubrieron.

## Specs

- **S1** — "eliminar el dependabot"
- **S2** — "eliminar totalmente loas coautorizaciones o colaboraciones que aparezcan de claude o dependabot, porque realmente no colaboraron nen nada, solo que mi agente erroneamente lo coloco no se porque."
- **S3** — Decisiones del dueño (2026-11-07), textual: "Sí, reescribir y forzar el push";
  "Re-autorear a Lucas Cardozo"; "creo que la solucion ya la aplique (configure para que
  no se pueda coautorizar ni commitear por nadie que no sea yo en git, fiojate si esta
  bien, y sino decime)"; "Solo leer, no escribir" (a `/home/lucas/.githooks`).
- **S4** — La regla del propio proyecto, ya escrita en las notas ODD:
  "NINGÚN commit lleva pie de atribución (nada de "Co-Authored-By" ni similar).
  El mensaje del commit explica qué cambió y por qué, y nada más."
- **S5** — El commit del bot (`7196a41`, el bump de `tauri-action`) pasa a ser de
  Lucas Cardozo, sin ningún trailer: el bot no colaboró en nada.
- **S6** — La prevención vive dentro del repositorio: no se escribe fuera de él.

## Tasks

| ID | Specs | Ruta | Commit |
|----|-------|------|--------|
| T1 | S2, S4 | inline | sin commit: el mapa de trailers y de identidades, en `## Log` |
| T2 | S2, S5 | inline | sin commit: reescritura de la historia (cambian los hashes) |
| T3 | S2, S5 | inline | sin commit: verificación de la reescritura, en `## Log` |
| T4 | S2 | inline | sin commit: force push de `main` y re-push de `v0.4.0` y `v0.5.1` |
| T5 | S1 | inline | `f4ad609` — `ci: Dependabot fuera del repositorio` |
| T6 | S1 | inline | sin commit: borrar la config de Dependabot del lado de GitHub |
| T7 | S3, S4, S6 | inline | `9ca4ccc` — `docs: la regla de atribución, y el check que la hace cumplir` |
| T8 | S3, S4, S6 | inline | `9ca4ccc` — el job `atribucion` de `checks.yml` |
| T9 | S2 | inline | `ca5b80c` — `docs(odd): los SHAs de los commits reescritos` |
| T10 | S3 | inline | sin commit: el informe de la configuración de git, en `## Log` |

## Log

### L1 — La petición (2026-11-07, textual)

> Buenas! quiero ahcer varias cosas en esta sesion:
>  - eliminar el dependabot
>  - eliminar totalmente loas coautorizaciones o colaboraciones que aparezcan de claude o dependabot, porque realmente no colaboraron nen nada, solo que mi agente erroneamente lo coloco no se porque.
>
> antes de hacer nada debes analizar todo y darme el plan para que lo autorice

### L2 — El análisis (2026-11-07)

- Dependabot: `.github/dependabot.yml` (v2, solo `github-actions`, semanal). Un único
  commit del bot en toda la historia: `7196a41` "ci(deps): bump tauri-apps/tauri-action
  from 0 to 1", autor `dependabot[bot]`, trailer `Signed-off-by: dependabot[bot]
  <support@github.com>`. Mergeado por `c4ab584` (commit propio, sin trailer). El PR #9
  ya está merged y su rama borrada; el remoto solo tiene `main`. Las *security updates*
  ya están `disabled` en el repositorio (la API de alertas devuelve 403).
- Co-autores: 24 commits con `Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>`,
  todos del 2026-11-02, en el bloque contiguo `2ebd571^..b11d219`. De ellos cuelgan 68
  commits: al reescribir cambian **93 hashes** (los 24 del bloque, el del bot y los 68
  que cuelgan de él) y **475 commits conservan el suyo**. La historia no es lineal
  (hay ramas laterales mergeadas después del bloque, con sus commits intactos).
- Tags: `v0.4.0` y `v0.5.1` caen dentro del rango reescrito (se re-pushean);
  `v0.3.0`, `v0.3.1`, `v0.3.2` y `fase4b-backup` quedan intactos.
- Causa raíz: no hay hook, ni template, ni `AGENTS.md`/`CLAUDE.md` en el repositorio. El
  trailer lo agrega la herramienta que escribe el commit (Claude Code firma por
  defecto). La regla "sin pies de atribución" vive solo en las notas ODD, que ningún
  agente lee.
- El hook global `/home/lucas/.githooks/commit-msg` ya existe (creado el 2026-11-06
  21:22, después de los 24 commits del 2-nov): borra los `co-authored-by:` que no son
  del dueño. Por eso no frenó esos commits, y por eso tampoco frena `Signed-off-by:`.
- `main` no tiene branch protection (`GET /branches/main/protection` → 404).
- Identidades en la historia: autores `Lucas Cardozo <lucasivancardozo27@gmail.com>` y
  `dependabot[bot]`; committers `Lucas Cardozo` y `GitHub <noreply@github.com>`
  (merges hechos por la UI). El check de CI va a mirar el **autor**, no el committer.

### L3 — Las decisiones (2026-11-07)

- Reescribir la historia y forzar el push (S3).
- El commit del bot se re-autorea a Lucas Cardozo (S5).
- La prevención va dentro del repositorio; `/home/lucas/.githooks` es solo lectura (S6).

### L4 — T1: el mapa (2026-11-07)

- 25 líneas de pie de atribución en toda la historia: 24 `Co-Authored-By: Claude Opus
  4.5 <noreply@anthropic.com>` y 1 `Signed-off-by: dependabot[bot] <support@github.com>`.
- Identidades: `Lucas Cardozo <lucasivancardozo27@gmail.com>` (autor y committer) y
  `GitHub <noreply@github.com>` (committer de los merges hechos por la UI).

### L5 — T2 y T3: la reescritura y su verificación (2026-11-07)

- Respaldo antes de tocar nada: `git bundle create .git/pre-clean-2026-11-07.bundle
  --all` (3,2 MB, historia completa) y el mapa de árboles y padres de los 568 commits.
- La reescritura se hizo en un clon aparte (`.rewrite-tmp`, dentro del repositorio), con
  `git filter-repo` y dos callbacks: uno saca las líneas `Co-Authored-By:` y
  `Signed-off-by:` de los mensajes (y solo de los mensajes que las tienen), y el otro
  pasa el commit del bot a Lucas Cardozo, autor y committer, con sus fechas.
- Verificación commit por commit (568 contra 568): **el árbol de cada commit es el
  mismo** (mismo SHA de árbol), los padres son los mismos traducidos por el mapa, los
  autores, los committers y las fechas son los mismos (salvo el commit del bot, que
  pasa a Lucas Cardozo), los mensajes son los mismos (salvo los pies de atribución) y
  los tags apuntan al commit equivalente. Cero diferencias. La historia nueva tiene un
  solo autor: `Lucas Cardozo <lucasivancardozo27@gmail.com>`.
- El mapa viejo→nuevo queda en `.git/rewrite-commit-map.txt` (fuera del árbol).

### L6 — T4: la publicación (2026-11-07)

- La historia reescrita se publicó primero en la rama `rewrite/limpieza-atribucion`
  (`23d01dc`), que no pisa nada: el guard de seguridad del harness bloquea el push con
  `--force`, y publicar en una rama nueva es la parte que no destruye.
- `main` sigue en `6b3bbf4` hasta que se mueva. El bundle `.git/pre-clean-2026-11-07.bundle`
  es el camino de vuelta: `git clone .git/pre-clean-2026-11-07.bundle vuelta` deja la
  historia vieja intacta.
