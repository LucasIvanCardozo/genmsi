# Lint: los avisos del formato en el documento de la CLI y del MCP

Rama `feature/lint-diagnostics` (worktree `../genmsi-lint2`), desde el HEAD
`5f1490c`. La decisión está tomada: el lint pasa a `status: "diagnostics"` para
un modelo con avisos, sin cambiar el esquema, sin cambiar la convención de los
códigos de salida y sin tocar `run-all`.

## Specs

- S1 (verbatim): "que el lint de la CLI y el del MCP digan lo mismo que la app
  cuando un modelo usa una entidad que no declara, sin romper nada de lo
  construido."
- S2 (verbatim): "el lint pasa a status \"diagnostics\" para un modelo con esos
  avisos, SIN cambiar el esquema del documento, SIN cambiar los códigos de
  salida, y el run-all no se toca."
- S3 (verbatim): "Revisa que un modelo con un almacén sin declarar NO produzca el
  aviso dos veces por dos caminos distintos: un solo aviso, con un solo texto."
- S4 (verbatim): "El código de salida del lint: sigue la convención que ya
  sigue el repo (mírala en docs/cli.md, sección de códigos de salida) y no
  inventes una. Un aviso no es un fallo: el lint no debe salir como si el modelo
  no cargara."
- S5 (verbatim): "El test del MCP que exige status == \"ok\" en los modelos de
  referencia: si con los cambios sigue siendo \"ok\" (debería ser, según la
  medición), no lo toques; si cae, actualízalo y explica por qué."
- S6 (verbatim): "La documentación: el significado de \"diagnostics\" en
  genmsi.lint/1, en docs/cli.md y en la documentación del MCP. El esquema no
  cambia (mismos campos, mismos tipos, mismos valores): no hay por qué subir la
  versión."
- S7 (verbatim): "cargo run --release -p genmsi-core --example oracle -- --runs 3
  (la comparación de los 25 modelos con sus bases debe seguir dando 20 ok / 5
  failed) y genmsi run-all fixtures/models debe decir exactamente lo mismo que
  antes: \"25 models: 20 ok, 5 failed\"."
- S8 (verbatim): "cargo test --workspace y el clippy en verde, con el número de
  tests subiendo."
- S9 (verbatim): "El commit en tu worktree, y me dices la rama."
- S10 (medición del HEAD, `odd/tasks/notas-lint.md`): con el `SaveValue` fuera
  de los kinds comprobados (82f6515), los 25 modelos de referencia tienen 2
  avisos, los dos en `Parcial 8-11-21`, que el CLI rechaza antes por su
  `QTable`. Ningún modelo alcanzable produce un aviso.

## Tasks

- T1 (S1, S2, S3) `genmsi-core`: el documento de lint suma los avisos del
  formato a las observaciones del compilador, sin repetir lo que el
  compilador ya dijo. `lint_diagnostics(program, model)` y
  `lint_document(program, model, path)`. Ruta: inline.
- T2 (S1, S4) `genmsi-cli`: `model::load` se queda con el modelo y `lint`
  arma el documento con él; el código de salida sigue siendo 3 (diagnostics),
  nunca 1. Ruta: inline.
- T3 (S1) `genmsi-mcp`: la herramienta `lint` devuelve el mismo documento y su
  descripción dice que también reporta los avisos del formato. Ruta: inline.
- T4 (S1, S3) `genmsi-app/engine`: el panel usa la misma suma del core, así
  que los tres clientes dicen lo mismo por construcción. Ruta: inline.
- T5 (S3, S5) Tests: el core dice un solo aviso para un almacén sin declarar;
  la CLI sale 3; el MCP dice "diagnostics" con un modelo en línea, y los 20
  modelos de referencia siguen en "ok". Ruta: inline.
- T6 (S6) Docs: `docs/cli.md` (tabla del documento, códigos de salida, sección
  `lint`), `docs/mcp.md` (la herramienta) y `docs/DECISIONS.md` (la decisión
  explícita de ensanchar el significado de `diagnostics` sin subir de versión).
  Ruta: inline.
- T7 (S7, S8) Verificación: los 25 `genmsi lint` byte a byte iguales, el
  `run-all` con la misma línea, el oráculo 20 ok / 5 failed, `cargo test
  --workspace` y clippy en verde. Ruta: inline.
- T8 (S9) Commit del work unit en `feature/lint-diagnostics`.

Un solo work unit: el comportamiento (un aviso, un texto) atraviesa los tres
clientes, y sus tests y sus docs son la misma unidad. La regla del escritor no
aplica (no hay dos unidades independientes).

## Log

- L1 (verbatim, el pedido): "Contexto: el bloque común de arriba.

  TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
    git worktree add ../genmsi-lint2 -b feature/lint-diagnostics
    cd ../genmsi-lint2

  TAREA: que el lint de la CLI y el del MCP digan lo mismo que la app cuando un
  modelo usa una entidad que no declara, sin romper nada de lo construido.

  LA DECISIÓN YA ESTÁ TOMADA (opción B-narrow): el lint pasa a status
  \"diagnostics\" para un modelo con esos avisos, SIN cambiar el esquema del
  documento, SIN cambiar los códigos de salida, y el run-all no se toca. La
  medición está en odd/tasks/notas-lint.md (ya en el repo).

  LO QUE YA ESTÁ HECHO (no lo repitas):
  - crates/genmsi-format/src/declaration.rs sabe qué entidades usa un bloque y
    cuáles declara el modelo: undeclared_entities(), declaration_warnings() y
    validate_declarations(). La app ya los suma a sus diagnósticos
    (crates/genmsi-app/engine/src/session.rs) y el DiagnosticsPanel los pinta.
  - Un save value NO se comprueban (el original los crea por uso); sí se
    comprueban Storage, Table y Function, que el original exige declarar.
  - El compilador ya reporta el almacén sin declarar como observación, y el lint
    sale con status diagnostics por eso (commit 0a4383f).
  - Con lo anterior, la salida de los 25 modelos de fixtures/models queda byte a
    byte igual: ninguno tiene un almacén, una tabla o una función sin declarar
    (el typo de Parcial 8-11-21 es de un almacén, pero ese modelo el CLI lo
    rechaza antes por otra cosa).

  QUÉ HACER:
  1. Que el documento del lint (genmsi.lint/1) de la CLI y el del MCP incluyan
     los avisos del formato (declaration_warnings) y digan status \"diagnostics\"
     cuando los hay. Ojo con el camino del core: el documento se construye en
     crates/genmsi-core/src/documents.rs, y el compilador ya tiene un camino
     parecido (la observación del almacén sin declarar). Revisa que un modelo con
     un almacén sin declarar NO produzca el aviso dos veces por dos caminos
     distintos: un solo aviso, con un solo texto.
  2. El código de salida del lint: sigue la convención que ya sigue el repo
     (mírala en docs/cli.md, sección de códigos de salida) y no inventes una.
     Un aviso no es un fallo: el lint no debe salir como si el modelo no
     cargara.
  3. El test del MCP que exige status == \"ok\" en los modelos de referencia: si
     con los cambios sigue siendo \"ok\" (debería ser, según la medición), no lo
     toques; si cae, actualízalo y explica por qué.
  4. La documentación: el significado de \"diagnostics\" en genmsi.lint/1, en
     docs/cli.md y en la documentación del MCP. El esquema no cambia (mismos
     campos, mismos tipos, mismos valores): no hay por qué subir la versión.
     Si al final el significado de un valor publicado cambia tanto que
     rompe a un cliente, dilo y lo decidimos.
  5. Comprueba con el oráculo que nada se movió:
     cargo run --release -p genmsi-core --example oracle -- --runs 3
     (la comparación de los 25 modelos con sus bases debe seguir dando 20 ok /
     5 failed) y genmsi run-all fixtures/models debe decir exactamente lo
     mismo que antes: \"25 models: 20 ok, 5 failed\".

  DONE CUANDO:
  - La CLI y el MCP dicen lo mismo que la app para un modelo con un almacén sin
     declarar, y la salida de los 25 modelos de referencia queda byte a byte
     igual.
  - cargo test --workspace y el clippy en verde, con el número de tests
     subiendo.
  - La documentación del lint al día en la CLI y en el MCP.
  - El commit en tu worktree, y me dices la rama."

- L2 (medición en el HEAD, antes de escribir): con el `SaveValue` ya fuera de
  los kinds (82f6515), `undeclared_entities` sobre los 25 modelos de
  `fixtures/models` da 2 avisos, los dos en `Parcial 8-11-21` (el `Storage`
  `acreditacion` y la `Table` `Tabla`), que el CLI rechaza antes por su
  `QTable` con un espacio en el nombre. Los 25 `genmsi lint` (humano y
  `--json`) y el `run-all` quedaron capturados como línea base en
  `/tmp/lint-baseline` para compararlos byte a byte.

- L3 (la forma del cambio): el documento de lint lo arma el core, así que la
  suma de los avisos vive en `genmsi-core::documents::lint_diagnostics`, y los
  tres clientes la usan: la CLI, el MCP y la app (que antes la tenía escrita a
  mano en `session.rs`, con la misma regla). El aviso duplicado se evita con
  la regla que la app ya usaba: si el compilador ya dijo el aviso del formato
  (el texto del formato va dentro del mensaje del compilador, con lo que la
  corrida hace), el del formato no se repite y gana el del compilador. El
  compilador solo reporta los almacenes; el formato revisa además tablas,
  funciones y los kinds que sólo se comprueban si el modelo los declara.

- L4 (verificado, T7):
  - los 25 `genmsi lint` (humano y `--json`) del worktree son **byte a byte**
    iguales a los de la línea base (`diff -r` sin diferencias);
  - `genmsi run-all fixtures/models` dice exactamente `25 models: 20 ok, 5
    failed`; con `--seed 1` la salida completa es idéntica a la del binario
    previo (sin semilla la corrida dibuja una aleatoria, así que el tiempo
    simulado de un modelo cambia entre corridas: no es de este cambio);
  - el oráculo (`cargo run --release -p genmsi-core --example oracle -- --runs
    3`) da los mismos 20 modelos con base y los mismos 5 rechazados;
  - `cargo test --workspace`: 339 tests en verde (333 antes: +3 del core, +2 de
    la CLI, +1 del MCP) y `cargo clippy --workspace --all-targets -- -D
    warnings` limpio. El repo ya tenía deuda de `cargo fmt` antes de este
    cambio y este cambio no agrega ninguna.

- L5 (decisión de esquema, S6): `genmsi.lint/1` se queda en `/1`. Los campos,
  sus tipos y el dominio de `status` (`ok` | `diagnostics`) no cambian; lo que
  se ensancha es el significado de `diagnostics`, que pasa de "observaciones
  del compilador" a "los hallazgos que no detienen el modelo". Se escribió en
  `docs/cli.md`, `docs/mcp.md` y en la decisión D16 de `docs/DECISIONS.md`.
  Lo único que un consumidor ve de más es que más modelos dicen `diagnostics`
  y salen con 3, que es justo lo que se quería. El tipo TypeScript sigue siendo
  `Array<string>` (sólo cambia el comentario del campo generado).

- L6 (commits, T8): `a899904` `feat(core): the lint says what the format
  found` lleva T1 (core), T2 (CLI), T3 (MCP), T4 (app), T5 (tests), T6 (docs)
  y T7 (verificación): un solo work unit, porque el comportamiento atraviesa
  los tres clientes y sus tests y sus docs son la misma unidad. El slice
  quedó en 511 líneas authored (290 de código y tests, 174 de este documento,
  47 de docs y tipos generados), apenas por encima de la guía de ~400: partir
  el comportamiento en dos commits habría dejado el primer commit sin los
  clientes que lo consumen, y el feature se cierra aquí.
