# Avisos en el lint: qué mide el contrato `genmsi.lint/1` y qué mediría si lo midiera

Trabajo en la rama `feature/lint-avisos` (worktree `../genmsi-lint`), desde el
HEAD `3ca6fb9`. Este documento es sólo de medición y propuesta: **no se
modificó el core ni ningún cliente**. Los "después" que se muestran salen de un
prototipo que se hizo, se midió y se tiró (ver §8).

## Lo que se pidió (verbatim)

> TAREA: medir el impacto de que el lint de la CLI y el del MCP reflejen los
> avisos de "el modelo usa una entidad que no declara" (la comprobación que
> acaba de integrarse, crates/genmsi-format/src/declaration.rs), y proponer el
> cambio con sus consecuencias.

## 1. La línea de partida, en una frase

La comprobación existe (`crates/genmsi-format/src/declaration.rs`), la app la
muestra, y **el lint no la dice**: `genmsi.lint/1` se arma en
`crates/genmsi-core/src/documents.rs:97` a partir de `Program::diagnostics`
(sólo observaciones del compilador), nunca a partir del modelo. Los tres
clientes se contradicen hoy: la app dice "diagnostics" para un modelo que la
CLI dice "ok".

| Cliente | Qué ve hoy | Dónde |
|---|---|---|
| CLI (`genmsi lint`) | `status: ok`, `diagnostics: none`, salida 0 | `crates/genmsi-cli/src/commands/lint.rs:54` |
| MCP (`lint`) | el mismo documento, byte a byte | `crates/genmsi-mcp/src/server.rs:218` |
| App (editor) | `status: "diagnostics"` + el aviso | `crates/genmsi-app/engine/src/session.rs:233` |

## 2. Los 25 modelos de referencia: la lista exacta de los avisos

Medido con `genmsi_format::undeclared_entities` sobre los 25 `.gmdl` de
`fixtures/models` (los 5 que el original rechaza se analizan igual, con
`parse_str`, para que su cuenta no dependa de si compilan).

**8 avisos en 2 de los 25 modelos.** Los otros 23 no tienen ninguno.

### `Parcial 8-11-21__ParcialMikelModelos.gmdl` — 7 avisos

| # | kind | clase | propiedad | bloque | nombre usado | lo que el modelo declara |
|---|---|---|---|---|---|---|
| 1 | Storage | `StorageUse` | `Storage` | 17 | `acreditacion` | `acreditacon` |
| 2 | SaveValue | `SaveValue` | `SaveValue` | 97 | `paradescongelar` | `CountVac` |
| 3 | SaveValue | `SaveValue` | `SaveValue` | 98 | `descongeladas` | `CountVac` |
| 4 | SaveValue | `SaveValue` | `SaveValue` | 42 | `paradescongelar` | `CountVac` |
| 5 | SaveValue | `Counter` | `SaveValue` | 99 | `CountPaciente` | `CountVac` |
| 6 | SaveValue | `SaveValue` | `SaveValue` | 96 | `descongeladas` | `CountVac` |
| 7 | Table | `Tabulate` | `NombreTabla` | 124 | `Tabla` | `Tabla1`, `vacunasfaltantes` |

### `TP10__tp10.gmdl` — 1 aviso

| # | kind | clase | propiedad | bloque | nombre usado | lo que el modelo declara |
|---|---|---|---|---|---|---|
| 1 | SaveValue | `Counter` | `SaveValue` | 27 | `CountEspera` | `CountPescador`, `IdPescCantidad`, `IdPescadorMayorPeso`, `MayorCantidad`, `MayorPeso` |

### Los avisos, contados por kind y por clase

- por kind: **SaveValue 6, Storage 1, Table 1**
- por clase: **Counter 2, SaveValue 4, StorageUse 1, Tabulate 1**

### El dato que decide el impacto: **1 de los 8 avisos es alcanzable hoy**

`Parcial 8-11-21` es uno de los 5 modelos que el original rechaza, y lo rechaza
por una razón *anterior* a la comprobación: la validación estructural lo tira en
el nodo 109 (`QTable: invalid block: definition name "tiempo demora" contains
whitespace or ':'`). La CLI, el MCP y la app lo rechazan **antes** de llegar a
cualquier aviso. Sus 7 avisos no los ve nadie, hoy ni después de ningún cambio
en el lint.

El único aviso que llega a un cliente es el de `TP10__tp10` (1 aviso,
`Counter`/`CountEspera`).

### Y ese único aviso es, según el original, discutible

La descripción de las clases del original (`Genmsi_20230711.backup/gClasses.xml`,
atributo `ClassDescrip`) exige declarar en **un** caso de todo el catálogo
(las clases que mencionan "declarar" son `StorageUse`, `FacilityUse` y
`FacilityInt`, y las dos últimas lo dicen sólo de una fila de espera
opcional):

- `StorageUse`: *"El Storage debe estar declarado (esta clase no lo declara)"* →
  el aviso 1 sí es un defecto real (el typo `acreditacion`/`acreditacon`).
- `Counter`: *"Numerador de transacciones. Usa una entidad saveValue."* → no pide
  nada. `InitiaSV`: *"Permite inicializar un SaveValue"* → inicializa, no
  declara la existencia. El `SaveValue` (kind del aviso de `TP10`) no tiene
  ninguna exigencia de declaración en el original.
- `Tabulate` y `Table` comparten la misma descripción: *"Define una tabla para
  ser tabulada"* → en el original el bloque `Tabulate` **define** la tabla que
  tabula, así que el aviso 7 es, con la regla del original, un falso positivo
  (y no por una casualidad: el motor ya emite para él el diagnóstico de
  runtime `TABULATE <name> has no TABLE definition`).

Y el comportamiento del motor lo confirma: el reporte del propio original para
`TP10` (`fixtures/outputs/TP10__tp10.report.txt:98`) trae `COUNTESPERA 2656`
como SAVEVALUE, creado por el bloque `Counter` (línea 34 del reporte:
`A_Counter1 SAVEVALUE CountEspera+,1`) **sin** ningún `InitiaSV` que lo declare.
El original crea la save value por uso, y nuestro motor hace lo mismo (el
oráculo da 61/64 métricas *match* para `TP10`, `docs/oracle-comparison.md:17`).

Es decir: la política `WhenDeclared` del formato (`declaration.rs:59`) marca el
`SaveValue` de `TP10` como aviso, pero el `SaveValue` es de los kinds que
comprueba el formato el único cuya regla el original no pide (la exige para el
`Storage`).

## 3. Qué cambia en la salida de la CLI

### 3.1 El documento `genmsi.lint/1`

El esquema **no cambia**: `status` ya admite `ok` y `diagnostics`
(`documents.rs:73`), `diagnostics` ya es un array de cadenas, y el código de
salida 3 ya existe para "el modelo corre pero el compilador dijo algo"
(`docs/cli.md:42`). Lo que cambia es **el significado** de `diagnostics`: de
"observaciones del compilador" a "observaciones del compilador + hallazgos del
formato".

Medido, `genmsi lint fixtures/models/TP10__tp10.gmdl`:

```
--- antes ---                          --- después (opción B) ---
model: fixtures/models/TP10__tp10.gmdl model: fixtures/models/TP10__tp10.gmdl
title: ConcursoPesca                   title: ConcursoPesca
status: ok                             status: diagnostics
blocks: 64                             blocks: 64
definitions: functions 5, ...          definitions: functions 5, ...
diagnostics: none                      diagnostics:
                                         - the model uses the SaveValue 'CountEspera' in
                                           the Counter block 27, and no InitiaSV block
                                           declares 'CountEspera' (the model declares:
                                           CountPescador, IdPescCantidad,
                                           IdPescadorMayorPeso, MayorCantidad,
                                           MayorPeso). Declare 'CountEspera' with a
                                           InitiaSV block, or use one of the declared names.
salida: 0                              salida: 3
```

Los otros 24 modelos no cambian de salida: 19 siguen en `status: ok` / salida 0,
y los 5 que el original rechaza siguen fallando con el mismo error y la misma
salida 1 (`Parcial 8-11-21` entre ellos, sin cambio: el aviso nunca se alcanza).

En `--json` el cambio es el mismo: `"status": "diagnostics"` y un elemento más
en el array `"diagnostics"`. El esquema, los nombres de los campos y los tipos
no cambian; **el significado de un campo cambia**, y D13
(`docs/DECISIONS.md:86`) dice que dentro de una versión mayor los campos
"conservan su nombre, tipo y **significado**". Estrictamente, ensanchar el
significado de `diagnostics` es un cambio incompatible y pediría
`genmsi.lint/2`, o una decisión explícita de reescribir el significado
documentado de `/1`.

Los tipos TypeScript **no cambian**: al regenerar
(`crates/genmsi-app/scripts/generate-types.sh`), `generated/LintDocument.ts`
sale byte a byte igual (`diagnostics: Array<string>`). (De paso: el
`generated/TraceEvent.ts` del repo ya estaba desfasado respecto del Rust; eso es
preexistente y no lo toca esta propuesta.)

### 3.2 `genmsi run-all fixtures/models`

Aquí está la sorpresa, y es el dato que más pesa en la decisión: **`run-all`
no consulta el documento de lint**. `sweep()`
(`crates/genmsi-cli/src/commands/run_all.rs:170`) carga, compila y **corre**;
el `status` de cada modelo es "corrió" o "falló". Por eso:

- **Opción B tal cual (lint cambia, `run-all` no)**: la salida es **idéntica a la
  de hoy**: `25 models: 20 ok, 5 failed`, salida 1, y el JSON
  `{"models_total":25,"models_ok":20,"models_failed":5}`. **El contrato del
  oráculo (20 ok / 5 failed) no se mueve.**
- **Opción B' (además, `run-all` cuenta aparte los modelos con avisos)**:
  `25 models: 19 ok, 1 with warnings, 5 failed`, salida 1 (sigue habiendo 5
  fallos), y el JSON gana `models_with_warnings: 1` mientras `models_ok` baja de
  20 a 19. Eso **sí** rompe lo construido:
  - `crates/genmsi-cli/tests/cli.rs:201-202` (la cadena exacta) y `:216`
    (`models_ok == 20`): **2 tests de integración rotos**;
  - `crates/genmsi-cli/src/commands/run_all.rs:241` (la línea del resumen en el
    test unitario): **1 test unitario roto**;
  - el snapshot `crates/genmsi-cli/tests/snapshots/cli__run_all_summary.snap:30`;
  - `docs/cli.md:145` y `:151` (la línea y su explicación).

  Y ojo: el nombre del campo `models_ok` cambiaría de significado (de "corrió" a
  "corrió limpio"), que es el mismo tipo de cambio incompatible que D13
  prohíbe dentro de una versión mayor.

## 4. Qué cambia en el MCP

El MCP devuelve el mismo documento por otra vía
(`crates/genmsi-mcp/src/server.rs:218` → `genmsi_core::lint_document`), y el
test de paridad (`crates/genmsi-cli/tests/documents_parity.rs:52`) obliga a que
los bytes sean los mismos que los de la CLI.

- **El documento**: idéntico al de la CLI, byte a byte. Sin cambios de schema.
- **El test que exige `status == "ok"`**:
  `crates/genmsi-mcp/tests/tools.rs:119` (`every_reference_model_reaches_the_tools`),
  línea 146: `assert_eq!(linted["status"], "ok", "{stem}: {linted}")`, sobre los
  20 modelos que el original acepta. Medido con el prototipo: **ese test falla
  y sólo ese** (1 de los 5 tests de `tests/tools.rs`), y falla en `TP10__tp10`:

  ```
  assertion `left == right` failed: TP10__tp10: {... "status":"diagnostics" ...}
    left: String("diagnostics")
   right: "ok"
  ```

  El arreglo es de una línea: que el test acepte `ok` o `diagnostics` para el
  modelo con avisos, y que compruebe que el aviso dice el nombre de la entidad.
  La otra mitad del test (`accepted == 20`, `refused == 5`) **no se mueve**: los
  5 rechazados siguen siendo tool errors `invalid_model`, con su `node`, `line` y
  `column` (medido: `Parcial 8-11-21` → `invalid_model`, nodo 109).
- **La descripción de la herramienta** (`server.rs:151`) dice hoy *"Validate a
  GenMSI model: it loads, validates and compiles"*; con la opción B debería
  decir que también reporta los avisos del formato (`docs/mcp.md:43`).

## 5. Qué cambia en la app

La app **ya está resuelta**: `diagnostics_of()`
(`crates/genmsi-app/engine/src/session.rs:233`) ya suma
`genmsi_format::declaration_warnings(model)` a las observaciones del compilador,
y el panel `DiagnosticsPanel.tsx:29` las muestra en ámbar ("El modelo compila con
observaciones"), al pie del `MonitorView` (`MonitorView.tsx:390`).

Medido con el motor de la app:

| Modelo | Lo que la app muestra hoy |
|---|---|
| `TP10__tp10` | `status=diagnostics`, 1 aviso (el mismo texto exacto que la CLI daría con la opción B) |
| `TP2__problema1` | `status=ok`, sin avisos |

Con la opción B, la app y la CLI dicen **lo mismo** del mismo modelo. Eso es el
argumento a favor de B. Y el cambio en la app es ninguno: el panel ya lo dice.
Dos detalles:

- El documento de lint de la app (`lint_model` → `engine::lint_document_bytes`)
  también llevaría los avisos, pero **ningún componente del frontend lo lee**:
  el adaptador `lint()` existe (`src/engine/adapter.ts:65`, `devBridge.ts:95`,
  `tauri.ts:82`) y ningún `.tsx` lo invoca. El panel se alimenta del estado de
  la sesión, no del documento de lint. Cero cambio visual.
- La trampa del doble conteo: `diagnostics_of` ya sumaba los avisos a mano, así
  que al mover la suma al core hay que quitarla de la app o cada aviso sale dos
  veces. Está medido: el prototipo tuvo que filtrar los duplicados
  (`session.rs:233-247`).

## 6. La propuesta, con su coste

### Opción A — el lint no cambia

- **Coste**: 0 ficheros, 0 tests, 0 docs. El contrato `genmsi.lint/1` queda
  como está publicado.
- **Lo que se pierde**: la CLI y el MCP no dicen nada del modelo con un aviso, y
  la app sí. Los tres clientes cuentan cosas distintas sobre el mismo fichero.
  Para el único caso alcanzable (`TP10`), `genmsi lint` dice "ok" y la app dice
  "el modelo compila con observaciones".

### Opción B — el lint pasa a `diagnostics`, sin cambiar esquema ni códigos de salida

- **Ficheros de código**: 7 (`genmsi-core/src/documents.rs` —la firma de
  `lint_document` gana `&GmdlModel` y `diagnostics` pasa de `&[String]` a
  `Vec<String>`—, `genmsi-cli/src/model.rs` —`Loaded` gana `model`—,
  `genmsi-cli/src/commands/lint.rs`, `genmsi-cli/tests/documents_parity.rs`,
  `genmsi-mcp/src/server.rs`, `genmsi-app/engine/src/lib.rs`,
  `genmsi-app/engine/src/session.rs`) + 1 test a arreglar
  (`genmsi-mcp/tests/tools.rs:146`).
- **Docs**: `docs/cli.md` (tabla de campos del lint, la línea del código 3 en
  `:42` y `:56-60`), `docs/mcp.md:43`, `docs/DECISIONS.md` (D13, si se decide
  ensanchar el significado de `diagnostics` sin subir de versión).
- **Lo que no cambia**: el esquema, los nombres de los campos, los tipos, los
  tipos TS, `run-all` (y su línea `25 models: 20 ok, 5 failed`), el contrato del
  oráculo (20 ok / 5 failed), la lista de los 5 modelos rechazados, el código de
  salida 1 de los que el original rechaza.
- **El coste de verdad**: el significado de `diagnostics` cambia dentro de
  `genmsi.lint/1`. Es el punto que hay que decidir explícitamente: o se sube a
  `genmsi.lint/2` (y entonces los consumidores que leen `genmsi.lint/1` se
  quedan atrás), o se acepta que `/1` significa "diagnostics: observaciones del
  compilador y avisos del formato" y se escribe en el documento. La segunda es
  más barata; la primera es la que D13 pide si se lee la regla al pie de la
  letra.

### Opción B' — B, y además `run-all` cuenta los modelos con avisos aparte

- **Todo lo de B**, más: `crates/genmsi-cli/src/commands/run_all.rs` (un campo
  nuevo en el documento `genmsi.run-all/1` y un tercer estado por modelo),
  `crates/genmsi-cli/tests/cli.rs` (2 tests), el test unitario de `run_all.rs`,
  el snapshot, `docs/cli.md:145`/`:151`.
- **El contrato del oráculo**: no se mueve. El 20/5 del oráculo es *qué modelos
  acepta el original*, no *qué dice nuestro lint*. Ese número vive en
  `crates/genmsi-core/tests/oracle.rs` (20 baselines válidos) y en
  `crates/genmsi-mcp/tests/tools.rs` (`accepted == 20`, `refused == 5`), y
  ninguno de los dos se entera de los avisos. Lo que sí se mueve es la línea
  documentada de `run-all` ("On `fixtures/models` the expected result is `20 ok,
  5 failed`", `docs/cli.md:151`), que es un artefacto de la CLI, no del oráculo.

### Opción B-narrow — B, pero sin comprobar el kind `SaveValue`

- **Idea**: dejar la comprobación donde el original la exige. Las descripciones
  de las clases del original sólo piden declarar el *Storage*; el `SaveValue` lo
  crea el propio uso (evidencia: el reporte del original para `TP10` trae
  `COUNTESPERA` sin `InitiaSV`), y el `Tabulate` del original *define* la tabla
  que tabula.
- **Medido**: con el kind `SaveValue` fuera de `KINDS`
  (`declaration.rs:59`), los avisos de los 25 modelos pasan de **8 a 2**
  (`Storage` 1 + `Table` 1, los dos en `Parcial 8-11-21`), y los avisos
  alcanzables pasan de **1 a 0**. (El kind `Table` también es dudoso —el
  `Tabulate` del original define la tabla—, pero no cambia nada hoy: su único
  aviso vive en un modelo que ningún cliente abre.)
- **Consecuencia medida**: la salida de `genmsi lint` para los 25 modelos de
  referencia es **byte a byte la de hoy** (`TP10` vuelve a `status: ok`, salida
  0; `Parcial 8-11-21` sigue con su error y salida 1), `run-all` sigue
  diciendo `25 models: 20 ok, 5 failed`, el test del MCP pasa sin tocarlo y los
  tipos TS no cambian. La maquinaria queda puesta (un modelo que use un
  `Storage` sin declarar sale con `status: diagnostics` y salida 3) sin tocar
  una sola salida publicada.

### Opción C — el lint falla con salida 1

- **Ficheros**: los de B, más la semántica del error.
- **Lo que rompe**:
  - el contrato del oráculo: el original **sí corre** `TP10` (tiene baseline,
    61/64 *match*), así que nosotros pasaríamos a rechazar un modelo que el
    original acepta → el 20/5 pasa a 19/6;
  - `crates/genmsi-mcp/tests/tools.rs` (`accepted == 20`) y el test de la CLI
    `every_reference_model_is_runnable_through_the_cli`
    (`crates/genmsi-cli/tests/cli.rs:273`);
  - `docs/cli.md:315-327` (la tabla de los modelos de referencia);
  - la semántica del código 1: significa "el modelo no carga, no valida, no
    compila o no corre" (`docs/cli.md:40`); un aviso no es ninguna de esas
    cuatro cosas, así que la salida 1 mentiría;
  - y contradice al propio formato: `declaration.rs` dice que el hallazgo "es un
    aviso, no un error de carga: el original abre un modelo que usa una entidad
    sin declarar".
- **Coste de fondo**: la app seguiría mostrando el caso como aviso mientras la
    CLI lo rechazaría: los tres clientes volverían a contradecirse, pero al revés.

## 7. La recomendación

**Opción B**, con el `run-all` intacto (B sin B'), y con una decisión explícita
sobre la política del `SaveValue` (§6, B-narrow).

Por qué:

1. **Es lo que hace que los tres clientes digan lo mismo.** La app ya dice el
   aviso; B hace que la CLI y el MCP digan lo mismo. A es la única opción que
   deja la contradicción; C la invierte.
2. **No mueve el oráculo.** B (sin B') no toca la línea `25 models: 20 ok, 5
   failed`, ni el 20/5 del oráculo, ni la lista de los 5 rechazados. Lo único
   que se mueve es `status` de un modelo (de `ok` a `diagnostics`) y su código
   de salida (de 0 a 3, que ya existe para exactamente ese caso).
3. **Cuesta un test.** 1 test del MCP, 1 línea. Ningún test de la CLI, ningún
   snapshot, ningún tipo TS.
4. **No le dice al original que está equivocado.** El original abre y corre
   `TP10`; un aviso no es un error, y el formato ya lo decidió así.

**La que menos rompe lo construido es B-narrow** (B + la política del `SaveValue`
acotada a lo que el original exige): la salida publicada de los 25 modelos
queda byte a byte igual, ningún test rota, y aun así el lint aprende a decir
"diagnostics" y sale 3 ante un modelo que use un `Storage` que no declara. Su
coste es una decisión de política (qué kinds se comprueban), no un contrato.

Si la decisión es que el aviso del `SaveValue` también se quiere contar, el
orden que propongo es: **B-narrow primero** (la maquinaria y el contrato, sin
ruido en la salida), y **B completo** en cuanto la política del `SaveValue`
esté decidida por escrito.

Lo que hay que decidir antes de escribir código (no lo decide esta nota):

1. ¿El `SaveValue` se comprueba o no? (B-narrow vs B). El original no lo pide.
2. ¿El `Tabulate` define la tabla que tabula (como dice su `ClassDescrip`) o la
   usa? Hoy el formato lo trata como uso y el motor emite además el diagnóstico
   de runtime `TABULATE <name> has no TABLE definition`.
3. ¿El significado de `diagnostics` dentro de `genmsi.lint/1` se ensancha (y se
   escribe en `docs/cli.md` y D13), o se sube a `genmsi.lint/2`?
4. ¿`run-all` cuenta los modelos con avisos aparte (B') o no? La respuesta
   decide si se toca la línea que el CI y la documentación tienen escrita.

## 8. Cómo se midió (para reproducirlo)

- Los 8 avisos: `genmsi_format::undeclared_entities` sobre los 25
  `fixtures/models` (`parse_str`, para que los 5 que el original rechaza
  entren en la cuenta). El propio repo ya tiene una prueba que fija el aviso de
  `Storage`
  (`crates/genmsi-format/tests/declarations.rs:157`: exactamente 1 aviso de
  `Storage` en los 25, el de `acreditacion`/`acreditacon`).
- Los "después": un prototipo (B) en este worktree, con el núcleo armando el
  documento de lint a partir del modelo y sumando los avisos, y una variante
  (B') con `run-all` contando los avisos aparte. Se compiló, se corrieron los
  25 modelos por la CLI y por el MCP, y se midieron las salidas y los tests que
  caen. **El prototipo se tiró después**: la rama `feature/lint-avisos` sólo
  contiene este documento.
- La suite completa, con el prototipo de B: pasa todo salvo
  `genmsi-mcp/tests/tools.rs` (1 test). Con el de B': además 2 tests de
  integración de la CLI, 1 test unitario y 1 snapshot.
- Las descripciones de las clases del original: `Genmsi_20230711.backup/gClasses.xml`,
  atributo `ClassDescrip` de `StorageUse`, `Counter`, `InitiaSV`, `Tabulate`.
- El reporte del original para `TP10`:
  `fixtures/outputs/TP10__tp10.report.txt` (líneas 34 y 98).
