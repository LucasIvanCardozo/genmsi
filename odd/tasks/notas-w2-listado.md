# Notas: el destino de la salida de fallo del `ALTER` y el operador "Invertir" de la llave lógica

Las dos diferencias del **listado del programa GPSS** que dejó el inventario
de la fase 7 en esta tanda: el destino de la salida de fallo del `ALTER` y el
tercer operador de la llave lógica, el que el manual llama "Invertir".

Cada una se midió contra el original, corriendo bajo Wine
(`scripts/wine-oracle/`): el listado que el propio original dejó y, para lo
que el listado no dice, el reporte.

El trabajo es de la **fase 7** (`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`).

## Specs

- **S1** — "EL DESTINO DE LA SALIDA DE FALLO DEL ALTER. El original escribe
  `ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1` y el nuestro
  `ALTER E UnGrupo,1,Param,99,Param,0`. En
  `crates/genmsi-core/src/gpss.rs` · `block_statements`, rama
  `BlockKind::Alter`. La sonda está en
  `scripts/wine-oracle/probes/AlterMiembro.gmdl`."
- **S2** — "EL OPERADOR 'INVERTIR' DE LA LLAVE LÓGICA. El manual
  (VISUALSIS.HLP) dice que `I` invierte el estado de la llave, y el motor
  trata cualquier operador que no sea 'Set' como un 'Reset' (y el listado
  sólo escribe `S` y `R`). En `crates/genmsi-core/src/program.rs` ·
  `Compiler::expand`, rama 'Logic'; y en `crates/genmsi-core/src/gpss.rs` ·
  `block_statements`, rama `BlockKind::Logic`. El listado tiene que escribir
  `LOGIC I 1` y la llave tiene que quedar invertida."
- **S3** — "Antes de cambiar nada, mide: corre el original con la sonda y
  compara su listado con el nuestro. El original manda."
- **S4** — "El punto 2 tiene dos partes: el texto del listado y la semántica
  (que la llave quede invertida). Fíjalas con tests que comprueben las dos
  cosas."
- **S5** — "Si el manual y el original discrepan, el original manda; y si el
  original no lo dice de forma inequívoca, NO lo inventes: déjalo como
  estaba y anótalo con lo que mediste."
- **S6** — "El GPSS no puede bajar de 62 de 62 y el oráculo no puede bajar de
  60 de 61. `cargo test --workspace` y `cargo clippy --workspace --all-targets
  -- -D warnings` en verde, con los tests subiendo."
- **S7** — "Un commit en la rama, con el estilo del repositorio. NINGÚN commit
  lleva pie de atribución."

## 1. El destino de la salida de fallo del `ALTER`

### La medida

La sonda `scripts/wine-oracle/probes/AlterMiembro.gmdl` tiene un `Alter` con
las dos salidas conectadas: `Salida1` ("Alteró") a `A_Test` y `Salida2` ("No
alteró") a `A_Terminate1`. El original, corrido con
`scripts/wine-oracle/run_oracle.sh`, escribe:

```text
A_Alter         ALTER    E       UnGrupo,1,Param,99,Param,0,A_Terminate1
```

y el nuestro escribía:

```text
A_Alter         ALTER           E UnGrupo,1,Param,99,Param,0
```

El banco `Alter`, cuya salida de fallo **no** está conectada, es la otra
medida: el original escribe `ALTER E ungrupo,1,param,0,otro,1`, con seis
operandos. El destino, entonces, sólo se escribe cuando la salida de fallo
está conectada.

La macro del original (`gmacroconfig.gmcfg`) lo dice en sus propios términos:
el operando `J` de la instrucción `ALTER` es la "Salida Alternativa, donde
se especifica el destino para la transaccion activa, en caso de que no haya
sido posible alterar ninguna transaccion", y el formato que el original
escribe cuando `J` no es nulo (`ALTER`) tiene un operando más que el que
escribe cuando no lo está (`ALTER6`).

### El código

`crates/genmsi-core/src/gpss.rs` · `block_statements`, rama
`BlockKind::Alter`: el destino de la salida de fallo ("No alteró", el puerto
`Salida2` de la clase) se escribe como el último operando de la sentencia,
y sólo cuando hay destino.

### La prueba

`crates/genmsi-core/tests/listado_pendientes.rs`:

- `the_alter_writes_the_destination_of_its_failure_output`: el `ALTER` con la
  salida de fallo conectada escribe
  `A_Alter ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1`.
- `the_alter_without_a_failure_output_keeps_its_six_operands`: sin salida de
  fallo conectada, el `ALTER` sigue escribiendo sus seis operandos.

### Lo que queda de esta sonda

El listado de la sonda `AlterMiembro` **todavía no coincide entero** con el
del original, y no por el `ALTER`: el original escribe los bloques en el
orden en que la transacción los visita, y la salida de fallo del `Alter` la
visita al final. El original lista `A_Terminate`, `A_Terminate2`,
`A_Terminate1`; el nuestro lista `A_Terminate`, `A_Terminate1`,
`A_Terminate2`. Es una diferencia del **orden de los bloques**
(`crates/genmsi-core/src/gpss.rs` · `flow_order`), anterior a este trabajo y
ajena a la lista de esta tanda: no se tocó.

## 2. El operador "Invertir" de la llave lógica

### Lo que dice el manual

`VISUALSIS.HLP`, la sección del bloque `LOGIC`: "Un bloque LOGIC es usado
para set, reset, o invertir el estado de una entidad llave lógica. [...] Si
el operador lógico es I, el estado de la llave lógica especificada por el
operando A es invertido. Esto significa que si esta en el estado set, esta
deberá ser cambiado a reset, y viceversa." Y la lista de operadores: "S. La
llave lógica es puesta en el estado 'set'. R. La llave lógica es puesta en el
estado 'reset'. I. La llave lógica es invertida."

### Lo que dice el original

- La lista `op_Swich` del propio original
  (`Genmsi_20230711/gmacroconfig.gmcfg`) tiene tres operadores:
  `S: Set`, `R: Reset` e `I: Invertir`. La lista `op_Logic_Swich`, que es la
  de las puertas, tiene dos: `LS: Set` y `LR: Reset`.
- El listado: con un `Logic` cuyo `Op` es `Invertir` (la sonda
  `LogicSet.gmdl` con el `Op` cambiado), el original escribe

  ```text
  A_Logic         LOGIC   I       1
  ```

- El reporte: en esa misma corrida, la sección `LOGICSWITCH` del original
  dice `1  1  0`, o sea que la llave quedó en **1**. La llave arrancaba en 1
  (el `InitialLS` de la sonda) y la investieron 60 transacciones: un número
  par de inversiones la deja como estaba. Si el operador fuera un reinicio,
  la llave quedaría en 0. La medición del reporte, y no sólo la del listado,
  dice que el original **invierte**.

### El código

- `crates/genmsi-core/src/program.rs` · `Compiler::expand`, rama `"Logic"`:
  el compilador lee el tercer operador (`Invertir`) y lo lleva en
  `LogicSpec::invert`. Antes leía cualquier operador que no fuera `Set` como
  un `Reset`.
- `crates/genmsi-core/src/gpss.rs` · `block_statements`, rama
  `BlockKind::Logic`: el listado escribe `I` cuando el bloque invierte.

El `LogicSpec` lleva la inversión en un campo propio (`invert`) y no como un
tercer variante de `LogicOp`, porque `crates/genmsi-core/src/report.rs`
(la rama `BlockKind::Logic` del reporte) hace un `match` **exhaustivo** sobre
`LogicOp`: agregarle un variante rompe la compilación de ese archivo, que es
de la otra tanda de trabajo en paralelo. Cuando ese archivo vuelva a estar
libre, lo natural es un `LogicOp::Invert` y el campo `invert` desaparece.

### La prueba

`crates/genmsi-core/tests/listado_pendientes.rs`:

- `the_logic_writes_the_inversion_as_the_original_does`: el listado de un
  `Logic` con `Op = Invertir` es `A_Logic LOGIC I 1`.
- `the_logic_keeps_the_letters_of_the_other_two_operators`: `Set` sigue
  escribiendo `S` y `Reset` sigue escribiendo `R`.

### Lo que queda pendiente: la semántica

La semántica (que la llave quede invertida) **no** está en el compilador: la
ejecuta `crates/genmsi-core/src/sim.rs`, en la rama `BlockKind::Logic` (la
línea 1334):

```rust
let state = spec.op == LogicOp::Set;
*self.entities.logic(&name) = state;
```

Ese archivo, y también `crates/genmsi-core/src/report.rs` y
`crates/genmsi-core/src/stat.rs`, son de la otra tanda de trabajo en
paralelo, así que **no se tocaron**. Lo que falta es:

1. **La semántica** (`crates/genmsi-core/src/sim.rs`, rama `BlockKind::Logic`):

   ```rust
   let state = if spec.invert {
       !*self.entities.logic(&name)
   } else {
       spec.op == LogicOp::Set
   };
   *self.entities.logic(&name) = state;
   ```

2. **El modo del reporte** (`crates/genmsi-core/src/report.rs`, rama
   `BlockKind::Logic`): el original escribe `LOGIC   I       1` en la columna
   `BLOCK_TYPE` y el nuestro escribe `LOGIC   R       1`, porque el reporte
   arma el modo con el `match` exhaustivo de `LogicOp`. Con el campo
   `invert`:

   ```rust
   mode = Some(if spec.invert {
       "I"
   } else {
       match spec.op {
           LogicOp::Set => "S",
           LogicOp::Reset => "R",
       }
   });
   ```

3. **La prueba de la semántica** está escrita y **espera** por lo de arriba:
   `the_logic_inverts_the_state_of_the_switch`, en
   `crates/genmsi-core/tests/listado_pendientes.rs`, con `#[ignore]` y la
   razón. La corrida del original que la respalda está más arriba: la llave
   queda en 1 después de 60 inversiones.

## 3. Lo que se midió y no se cambió

Tres cosas más que se midieron con el original y que **no** se tocaron,
porque no estaban en la lista de esta tanda. Quedan anotadas con su medida
para quien las pida.

### El `ALTER` con "Modo = Todos"

El banco `Alter` usa `Modo = Algunos`. Con `Modo = Todos`, el original
escribe el operando del límite como `ALL`:

```text
A_Alter         ALTER    E       ungrupo,ALL,param,0,otro,1
```

y el nuestro escribe `A_Alter ALTER E ungrupo,1,param,0,otro,1`. La macro
del original lo dice: el operando es el valor de `C` (la cantidad), salvo
que `B` (el modo) sea `Todos`, en cuyo caso el original escribe `ALL`. El
cambio es de una línea en la misma rama `BlockKind::Alter` de
`crates/genmsi-core/src/gpss.rs` (el `spec.count` se reemplaza por `ALL`
cuando el modo es `Todos`), pero hace falta el modo en el `AlterSpec`, que
hoy no se compila.

### El `InitialLS` con `Valor` = "Invertir"

La clase `InitialLS` declara el valor inicial de la llave y su propiedad
`Valor` también es de la lista `op_Swich`. El original escribe:

| `Valor` | el original escribe |
|---|---|
| `Set` | `INITIAL LS1` |
| `Reset` | `INITIAL LR1` |
| `Invertir` | `INITIAL LS1` |

O sea que el original lee el valor inicial como "si no es `Reset`, es `set`",
y el nuestro lo lee al revés (si no es `Set`, es `reset`): con
`Valor = Invertir` el nuestro pondría la llave en `reset` y el original la
pone en `set`. La regla del original está medida; el cambio es de una línea
en `crates/genmsi-core/src/program.rs` · `Compiler::expand`, rama
`"InitialLS"` (hoy `is_some_and(|op| op.eq_ignore_ascii_case("Set"))`), y
está fuera de la lista de esta tanda.

### El orden de los bloques de la sonda `AlterMiembro`

Está descrito arriba, en la primera parte: el original lista la salida de
fallo del `Alter` al final y el nuestro la lista donde está en el modelo. Es
una diferencia de `flow_order` (`crates/genmsi-core/src/gpss.rs`), anterior a
este trabajo.

### La respuesta vieja del original para el banco `Tabulate`

En `/tmp/bench-out-new` la respuesta del original para el banco `Tabulate`
era de una versión anterior del banco (la que no declaraba la tabla): el
original terminaba con "ERROR!! Se intentó actualizar el valor de una tabla
inexistente" y su listado no traía la línea `Tabla TABLE`. Con el banco de
hoy, el original corre y escribe esa línea. Se volvió a correr el original
sobre el banco actual y se refrescaron `Tabulate.gpss.txt` y
`Tabulate.report.txt`; con eso el GPSS pasa de 61 de 62 a **62 de 62** y las
métricas de 60 de 61 a **61 de 61**.

## 4. Las cifras

- **GPSS**: 62 de 62 bancos con respuesta del original coinciden
  (`bash scripts/wine-oracle/compare-benches.sh scripts/wine-oracle/benches
  /tmp/bench-out-new`). Los cuatro bancos sin respuesta del original
  (`MFunction`, `MFunctionDist`, `MultifurcacionP`, `RemoveNumber`) siguen sin
  respuesta.
- **Métricas**: 61 de 61 bancos con reporte del original coinciden en todas
  sus métricas (`cargo run --release -p genmsi-core --example
  compare-benches -- scripts/wine-oracle/benches /tmp/bench-out-new --runs
  3`).
- **Diferencial**: los 66 bancos y las 33 sondas se listaron con el código de
  antes (`6172b6d`) y con el de ahora: la única diferencia es la del `ALTER`
  de la sonda `AlterMiembro`, que es la que se quería. Ni un banco cambia.
- **Tests**: `cargo test --workspace` en verde: 376 tests en verde y uno en
  `#[ignore]`, que son los cinco nuevos de
  `crates/genmsi-core/tests/listado_pendientes.rs` (cuatro en verde y el de
  la semántica, a la espera de `sim.rs`).
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` en
  verde.
