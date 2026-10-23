# Notas: la puerta que no abre y la inversión de la llave lógica

Las dos diferencias que el **inventario de la fase 7** dejó para el motor
(`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`, L5 y T11): la puerta del
`GateL` que no abre y el tercer operador de la llave lógica, el que el manual
llama "Invertir".

Las dos se midieron contra el original, corriendo bajo Wine
(`scripts/wine-oracle/run_oracle.sh`), y el original manda.

## Specs

- **S1** — "LA PUERTA QUE NO ABRE. La fila de la puerta del banco `GateL`
  tiene que salir `0 0 0`, como en el reporte del original: el original no
  cuenta la entrada del intento fallido y cuenta los intentos en la llave
  que la puerta mira."
- **S2** — "LA INVERSIÓN DE LA LLAVE LÓGICA. El tercer operador de la llave,
  `I`, invierte el estado: el original escribe `LOGIC I 1` y su reporte deja
  la llave invertida."
- **S3** — "Antes de cambiar nada, mide: corre el original con el banco
  `GateL` y con una sonda de la llave lógica y compara su reporte con el
  nuestro. El original manda."
- **S4** — "Un test por cosa, que fije el número exacto de la fila de la
  puerta (`0 0 0`) y el estado de la llave tras un número impar y par de
  inversiones."
- **S5** — "Si un punto necesita tocar otro archivo que no está en tu
  superficie, NO lo hagas: anótalo aquí con el parche exacto y el test
  escrito."
- **S6** — "El oráculo no puede bajar de 61 de 61. `cargo test --workspace` y
  `cargo clippy --workspace --all-targets -- -D warnings` en verde."

## 1. La puerta que no abre

### La medida

Corrida nueva del original sobre el banco `GateL` (`/tmp/w5/oracle/`):

```text
 LINE    ENTRY CURRENT RETRY            LOC  BLOCK_TYPE
         COUNT   COUNT
    1       62      62     0              1  GENERATE        60,30
    2        0       0     0          GateL  GATE    LS      1
    3        0       0     0    A_Terminate  TERMINATE
    4        2       1     0              4  GENERATE        3600
    5        1       0     0              5  TERMINATE       1

LOGICSWITCH               VALUE      RETRY
  1                          0         61
```

La fila de la puerta es `0 0 0`: el intento fallido **no** es una entrada de
la puerta, y los intentos que fallan se cuentan en la llave que la puerta
mira (aquí, 61 intentos en la llave `1`, que es lo que dice la sección
`LOGICSWITCH` del original). La puerta que sí abre sí cuenta su entrada
(el banco `GateS` del original: `62 0 0` en la fila de la puerta).

### El estado del código

**Esta parte ya estaba hecha** cuando llegó la unidad de trabajo, en
`e5610e2` ("fix(core): the shape of the report is the original's"), que es
trabajo en paralelo: `crates/genmsi-core/src/sim.rs` · rama
`BlockKind::Gate` ya excluía la puerta del conteo general de entradas, ya
contaba el intento fallido en la entidad que la puerta mira
(`fn count_gate_retry`) y ya no contaba el intento fallido como reintento del
bloque.

Comprobado sobre este mismo banco antes de tocar nada:

```text
    2        0       0     0          GateL  GATE    LS      1
LOGICSWITCH               VALUE      RETRY
  1                          0         59
```

Es decir: la fila `0 0 0` que el padre pedía ya salía, y el oráculo ya
comparaba el banco `GateL` en 2 de 2 métricas. El L5 del inventario de la
fase 7 ("una sola diferencia de comportamiento queda en todo el inventario: la
puerta del `GateL` que no abre") y su T11 ("pendiente") quedaron atrás de
`e5610e2`; la L5 y el T11 habría que actualizarlos (archivo del inventario,
fuera de esta superficie).

Lo que sí faltaba era **fijar la regla con un test**, para que la fila no
vuelva a moverse. Está en
`crates/genmsi-core/tests/puerta_y_logica.rs`:

- `the_gate_that_does_not_open_counts_nothing`: la fila de la puerta sale
  `0 0 0` (entradas, current y retry del bloque), comprobado sobre el texto
  del reporte y sobre los contadores del bloque.
- `the_attempt_that_fails_the_gate_is_counted_on_the_switch`: los intentos que
  fallan se cuentan en la llave que la puerta mira, no en el bloque.

### Lo que queda de la puerta: el `CURRENT` del bloque anterior

Medido, y **no** cambiado (está fuera de la superficie de esta unidad y
mueve los snapshots de la CLI, que son de otra unidad).

La transacción que espera en la puerta que no abre **sigue contando en el
bloque del que viene**, no en la puerta. Lo dice el reporte del original del
fichero `fixtures/outputs/TP6__Clase_6_1_PuertaBanco.report.txt`:

```text
    1       14       1     0              1  GENERATE        90,30,2700
    2       13      13     0        A_Queue  QUEUE           qAfuera
    3        0       0     0          GateL  GATE    LS      Puerta
```

La cola (el bloque anterior a la puerta) tiene 13 entradas y **13** de
`CURRENT`: las trece transacciones que esperan en la puerta siguen en la
cola. En el banco `GateL` lo mismo, en el bloque anterior (el generador): el
original dice `62 62` y el nuestro `60 1`.

Es la misma regla que el `SEIZE` ya tiene en
`crates/genmsi-core/src/sim.rs` (brazo `BlockKind::Seize`): "la transacción
retrocede al bloque del que vino". El parche, para la unidad que pueda
aplicarlo:

```rust
// crates/genmsi-core/src/sim.rs, rama BlockKind::Gate, brazo `else`
// (la puerta no abre y la transacción espera):
                    // The transaction that waits for the gate is still in the
                    // block it came from: the original keeps it there
                    // (`A_Queue  QUEUE  qAfuera  13 13` with the gate shut).
                    if !resume
                        && let Some(previous) = self.previous_block_of(xact)
                    {
                        self.block_stats[previous].current += 1;
                    }

// crates/genmsi-core/src/sim.rs, rama BlockKind::Gate, brazo `if pass`
// (la puerta abre y la transacción sale):
                    // It leaves the block it was waiting in.
                    if let Some(previous) = self.previous_block_of(xact)
                        && self.block_stats[previous].current > 0
                    {
                        self.block_stats[previous].current -= 1;
                    }
```

Ojo: este parche cambia la columna `CURRENT` de los reportes y **rompe los
snapshots de la CLI** (`crates/genmsi-cli/tests/snapshots/cli__simulate_report.snap`
y `cli__simulate_json.snap`, el modelo `TP6__Clase 6_1_PuertaBanco`), que son
de otra unidad de trabajo. Por eso no se aplicó aquí.

Comprobado en una copia del repositorio fuera del worktree: con el parche, la
fila de la cola del banco `GateL` pasa de `9 0 0` a `9 9 0` (la del original
`TP6__Clase_6_1_PuertaBanco` es `13 13 0`), la del `GENERATE` del banco
`GateL` pasa de `60 1 0` a `60 60 0` (la del original es `62 62 0`), y
fallan exactamente los dos snapshots de la CLI:

```diff
--- crates/genmsi-cli/tests/snapshots/cli__simulate_report.snap
-    2        9       0     0        A_Queue  QUEUE           qAfuera
+    2        9       9     0        A_Queue  QUEUE           qAfuera
--- crates/genmsi-cli/tests/snapshots/cli__simulate_json.snap
-      "current": 0,
+      "current": 9,
```

## 2. La inversión de la llave lógica

### La medida

El manual (`VISUALSIS.HLP`, bloque `LOGIC`) dice del tercer operador: "Si el
operador lógico es I, el estado de la llave lógica especificada por el operando
A es invertido". La lista `op_Swich` del propio original
(`Genmsi_20230711.backup/gmacroconfig.gmcfg`) tiene tres operadores: `S`
(Set), `R` (Reset) e `I` (Invertir).

Seis sondas nuevas, corridas con el original
(`bash scripts/wine-oracle/run_oracle.sh <modelo> <destino>`; modelos en
`/tmp/w5/probes/`, respuestas en `/tmp/w5/oracle/`). Todas son el mismo
modelo —`INITIAL LS1` (o nada, y entonces la llave arranca deshecha), un
`Generate` con `Cantidad` exacta y un `LOGIC I 1`— y lo que se compara es el
valor de la llave en la sección `LOGICSWITCH` del reporte:

| sonda | la llave arranca | inversiones | el original | un `Set` | un `Reset` |
|---|---|---:|---:|---:|---:|
| `LogicInvertir1` | puesta | 1 | **0** | 1 | 0 |
| `LogicInvertir2` | puesta | 2 | **1** | 1 | 0 |
| `LogicInvertir3` | puesta | 3 | **0** | 1 | 0 |
| `LogicInvertir4` | puesta | 4 | **1** | 1 | 0 |
| `LogicInvertirSinInicial1` | deshecha | 1 | **1** | 1 | 0 |
| `LogicInvertirSinInicial3` | deshecha | 3 | **1** | 1 | 0 |

El original **invierte**: un número impar de inversiones deja la llave en el
estado contrario al de partida y un número par la deja como estaba. Ni un
`Set` ni un `Reset` explican la tabla entera: el `Set` falla en las sondas
`1` y `3` (la llave puesta), y el `Reset` falla en las sondas `2`, `4`,
`SinInicial1` y `SinInicial3`.

El listado del original escribe `A_Logic         LOGIC   I       1` en las
seis sondas, y el nuestro ya escribía eso (el listado es de
`crates/genmsi-core/src/gpss.rs`, de otra unidad).

### El código

`crates/genmsi-core/src/sim.rs` · rama `BlockKind::Logic`: el motor leía
cualquier operador que no fuera `Set` como un `Reset`, y la inversión no
tenía efecto. Ahora:

```rust
let state = if spec.invert {
    !*self.entities.logic(&name)
} else {
    spec.op == LogicOp::Set
};
*self.entities.logic(&name) = state;
```

El campo `invert` de `LogicSpec` lo trae el compilador
(`crates/genmsi-core/src/program.rs` · `Compiler::expand`, rama `"Logic"`) y
el listado lo usa (`crates/genmsi-core/src/gpss.rs`, rama `BlockKind::Logic`).

### La prueba

`crates/genmsi-core/tests/puerta_y_logica.rs`:

- `the_logic_inverts_the_switch_after_an_odd_number_of_inversions`: la llave
  arranca deshecha y una inversión la deja puesta (`1`), como en la sonda
  `LogicInvertirSinInicial1` del original.
- `the_logic_inverts_the_switch_after_an_even_number_of_inversions`: la llave
  arranca puesta y dos inversiones la dejan puesta (`1`), como en la sonda
  `LogicInvertir2`.
- `the_logic_inverts_the_switch_that_starts_set`: la llave arranca puesta y
  una inversión la deja deshecha (`0`), como en la sonda `LogicInvertir1`
  (esto es lo que separa la inversión de un `Set`).
- `the_logic_keeps_the_meaning_of_the_other_two_operators`: `Set` y `Reset`
  siguen siendo lo que eran.

## 3. Lo que queda en otras unidades

### El modo de la fila `LOGIC` del reporte

El original escribe `LOGIC   I       1` y el nuestro `LOGIC   R       1`
(la fila del reporte, no el listado). Está en `crates/genmsi-core/src/report.rs`,
que es de otra unidad. El parche:

```rust
// crates/genmsi-core/src/report.rs, rama BlockKind::Logic
        BlockKind::Logic(spec) => {
            // `LOGIC   R       Molinete` / `LOGIC   S       Molinete`.
            mode = Some(if spec.invert {
                "I"
            } else {
                match spec.op {
                    LogicOp::Set => "S",
                    LogicOp::Reset => "R",
                }
            });
            vec![spec.switch.to_string()]
        }
```

### El test ignorado de `listado_pendientes.rs`

`crates/genmsi-core/tests/listado_pendientes.rs` tiene el test
`the_logic_inverts_the_state_of_the_switch`, con `#[ignore]` y la razón "the
engine does not invert the switch yet: sim.rs is another work unit's file".
El motor ya invierte, así que el `#[ignore]` se puede quitar, **pero** el test
tal como está no vale: su generador no tiene `Cantidad`, y en los 100
unidades de tiempo de la corrida produce 100 transacciones, o sea 100
invenciones (un número par): la llave termina deshecha y la aserción falla
igual. Con una sola inversión el test pasa. El parche:

```rust
// crates/genmsi-core/tests/listado_pendientes.rs, fn logic_model:
// el generador produce una sola transacción, y la inversión es una.
         node(
             1,
             "Generate",
             None,
             &[("Salida1", Some("A_Logic"))],
-            &[("Media", "1"), ("Modificador", "0")],
+            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "1")],
         ),

// y el test, sin el #[ignore]:
-#[ignore = "the engine does not invert the switch yet: sim.rs is another work unit's file"]
 fn the_logic_inverts_the_state_of_the_switch() {
```

Comprobado en esta unidad, en una copia del repositorio fuera del worktree:
con el `#[ignore]` quitado y el `Cantidad` de uno, los cinco tests de
`listado_pendientes.rs` pasan (`cargo test --test listado_pendientes` →
"5 passed; 0 failed; 0 ignored").

## 4. Las cifras

- **Oráculo**: 61 de 61 bancos con reporte del original coinciden en todas
  sus métricas (`cargo run --release -p genmsi-core --example compare-benches
  -- scripts/wine-oracle/benches /tmp/bench-out-new --runs 3`). El `GateL`
  está en 2 de 2.
- **Diferencial**: los 66 bancos, con el código de antes (`de74fae`, en una
  copia en `/tmp`) y con el de ahora: los 66 reportes salen **idénticos**.
  Los seis modelos de la inversión salen distintos, que es lo que se quería.
- **Tests**: `cargo test --workspace` en verde (51 ejecutables de test, 0
  fallos; queda un `#[ignore]`, el de `listado_pendientes.rs` del punto 3).
  Los seis nuevos de `crates/genmsi-core/tests/puerta_y_logica.rs` en verde.
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` en
  verde.

## 5. Lo que queda: la traza no sigue la misma regla que el reporte

La regla del original que este trabajo unit aplica al **reporte** ("la
transacción que espera en la puerta que no abre sigue contando en el bloque
del que vino") **no está en la traza**. Y el reproductor de la app, que
reconstruye el estado de la corrida a partir de la traza, la nota.

### El síntoma

El test del front `crates/genmsi-app/tests/replay.test.ts` ("the replay of the
reference model ends in the state the report publishes") falla con:

```text
AssertionError: the blocks differ
  actual:   [ 4, 13 ]
  expected: []
```

Los bloques 4 y 13 son la puerta (`GateL`) y la cola (`A_Queue`) del modelo de
referencia `fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl`.

### La causa, medida

El reporte y la traza cuentan la misma transacción bloqueada en dos sitios
distintos:

| | reporte (fila de la cola) | reporte (fila de la puerta) | traza |
|---|---|---|---|
| el original | `13 13 0` | `0 0 0` | — |
| el nuestro | `9 9 0` | `0 0 0` | 9 transacciones esperando en la puerta (bloque 4), 0 en la cola |

El reporte cuenta la transacción en la cola (que es lo que dice el original) y
la traza la cuenta en la puerta. La traza emite, para la transacción
bloqueada: `cola: enter`, `cola: leave -> 4`, `puerta: enter`, `puerta: wait`.
El reproductor cuenta occupancy por `enter`/`leave`, así que la transacción le
queda en la puerta.

El motor ya sabe la regla: la rama de la puerta que no abre hace
`self.block_stats[previous].current += 1` (`crates/genmsi-core/src/sim.rs`,
`BlockKind::Gate`), y el `SEIZE` que no toma la facilidad hace lo mismo
(`BlockKind::Seize`). Lo que no hace ninguna de las dos es la contraparte en
la traza.

### No es una regresión de este trabajo

Control diferencial, con el mismo front y los binarios de antes y después del
cambio del orden del listado (`1b6f283`):

| binarios | resultado |
|---|---|
| de `d07f92b` (antes del cambio del listado) | `actual: [4, 13]`, `expected: []` |
| de `4bdca5d` (después) | `actual: [4, 13]`, `expected: []` |

El fallo es anterior al cierre de la fase 7 y no depende del orden del
listado. Lo detecta el propio test de la app, que compara el estado final del
reproductor con el reporte del motor.

### Lo que hay que decidir

Arreglarlo toca el significado de los eventos `enter`/`leave` de la traza para
los bloques que retroceden (`Gate`, `Seize`), y el contrato `genmsi.trace/1`
promete que "los campos existentes conservan su nombre, tipo y significado". Las
dos salidas posibles:

1. **El motor**: en la rama de espera, emitir los eventos que expresen el
   retroceso (la transacción sale del bloque de la puerta y vuelve al bloque
   del que vino). Rompe los dos snapshots de la CLI que este trabajo unit ya
   sabía que se rompían, y cambia lo que la reproducción muestra.
2. **La app**: que el reproductor interprete el `wait` de un bloque que
   retrocede como "la transacción sigue en el bloque anterior". No toca el
   contrato de la traza, pero cambia la proyección de la app.

Ninguna de las dos se aplica aquí: la primera cambia un contrato publicado y la
segunda cambia lo que la reproducción enseña. La decisión es de quien
coordina.
