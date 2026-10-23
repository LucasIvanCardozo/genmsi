# Avisos del editor y del validador: lo que el modelo usa y no declara

Trabajo en la rama `feature/editor-avisos` (worktree `../genmsi-editor`). Este
documento es sólo de hallazgos: el documento de fase 6 no se tocó.

## Lo que se pidió (verbatim)

> TAREA: que el editor y el validador digan cuando un modelo usa algo que no
> declaró, en vez de dejarlo pasar en silencio.
>
> - Un almacén (StorageUse) sin un StorageDef que lo declara: el motor hoy le
>   pone una capacidad de 1.000.000 y todo se atiende en paralelo. Esa sesión
>   decide el comportamiento del motor; tú haces que el MODELO avise.
> - Lo mismo con las otras entidades que se declaran en su propia clase:
>   StorageDef, Table, QTable, Function, FunctionDist, VariableA, FunctionExpon,
>   Variable, InitiaSV, InitialLS, FacilityInt/FacilityUse/Seize/Release/Preempt
>   (la facility que usan), Queue, Link/Unlink/JoinMe/JoinNumber (la cadena).
>
> - Hazlo en la capa que corresponda (el validador del formato, que es donde
>   vive la comprobación estructural; si la app tiene su propio sitio para los
>   avisos, que lo muestre).
> - El aviso debe decir qué se declara y cómo se arregla, no sólo que falta.
> - No cambies el comportamiento del motor: eso lo decide la otra sesión.

## La regla del original

El original la dice en la descripción de su propia clase `StorageUse`
(`Genmsi_20230711.backup/gClasses.xml`, atributo `ClassDescrip`):

> Las transacciones ingresan a un storage por un sierto tiempo y salen. Si el
> storage esta lleno, aguardan haciendo cola.
> **El Storage debe estar declara (esta clase no lo declara).**
> - Se debe utilizar otro módulo que defina la capacidad del Storage.

Y su editor no deja escribir el nombre: el selector de entidades de una
propiedad `EntityUse` sólo ofrece las entidades que el modelo declara.

En el formato, esa regla es legible sin interpretar nada: cada propiedad del
modelo trae su tipo y su subtipo, y el tipo dice si la propiedad **declara**
una entidad (`EntityDef`) o si la **usa** (`EntityUse`). Los nombres de las
clases que declaran, según el catálogo (`crates/genmsi-format/src/catalogue.rs`)
y los propios modelos de referencia:

| Entidad | Quién la declara |
| --- | --- |
| `Storage` | `StorageDef` |
| `Table` | `Table` |
| `QTable` | `QTable` |
| `Function` | `Function`, `FunctionDist`, `FunctionExpon`, `Multifurcacion`, `MultifurcacionP` |
| `Variable` | `Variable`, `VariableA`, `VariableN` |
| `SaveValue` | `InitiaSV` |
| `LogicSwich` | `InitialLS` |
| `Label` | `Match` |

## La decisión que se took (y por qué)

**1. Es un aviso, no un error al cargar.** El original abre un modelo que usa
un almacén sin declarar (el motor le inventa la capacidad y sigue), así que el
cargador (`genmsi_format::load` → `validate`) **no** falla por esto. La
comprobación estricta vive aparte, en `validate_declarations`, para el que
quiera rechazar ese modelo. El motor no se toca: su comportamiento lo decide
la otra sesión.

**2. Dos políticas, según lo que el modelo haga con esa clase de entidad.**

- `Storage`, `Table` y `Function`: se comprueban **siempre**. Una entidad de
  estas sin definir no significa nada (un almacén sin capacidad, una tabla
  sin frecuencias, una función sin distribución), y es justo el caso que
  pide el encargo.
- `SaveValue`, `LogicSwich`, `Label`, `QTable` y `Variable`: se comprueban
  **sólo cuando el modelo declara esa clase de entidad en otra parte**. Si el
  modelo no declara ningún save value, es que no los usa como entidades
  declaradas (GPSS los crea al vuelo); si declara cinco y usa un sexto
  nombre, eso sí es un error de dedo.

**3. Las entidades que GPSS crea al usarse no se avisan.** `Queue`,
`Facility`, `UserChain` (la cadena), `TransacGroup`, `NumericGroup`,
`Parameter` y `RandomSeed` no las declara ninguna clase: en GPSS nacen con el
primer uso. La evidencia está en los modelos de referencia: **23 de los 25**
usan colas que nadie declara (sólo las declara un `QTable`, y muy pocos
modelos tienen uno). Avisar de eso sería enterrar el hallazgo de verdad (el
typo del almacén) bajo decenas de avisos falsos. La regla está en una tabla de
una línea por clase de entidad (`KINDS`, en `declaration.rs`): añadir `Queue` es
cambiar `Policy::WhenDeclared` por `Policy::Required` y ver lo que pasa.

**4. Un valor que no es un nombre plano no se comprueba.** `*Cuadra`,
`P:Param` y los marcadores como `[storage]` se calculan en tiempo de ejecución
o son marcadores: no hay ningún nombre contra el cual comparar una
declaración. (El modelo `Problema 1__semaforo-1` usa el almacén `*Cuadra` y
no se toca.)

## Lo que se hizo

### `crates/genmsi-format/src/declaration.rs` (nuevo)

La comprobación estructural, en la capa del formato:

- `UndeclaredEntity`: qué bloque usa qué entidad, de qué clase, y qué
  nombres de esa clase declara el modelo.
- `undeclared_entities(model) -> Vec<UndeclaredEntity>`: el diagnóstico.
- `declaration_warnings(model) -> Vec<String>`: los mensajes, listos para
  mostrar.
- `validate_declarations(model) -> Result<(), FormatError>`: la forma
  estricta (el primer uso sin declarar es un error, con el nodo del bloque).

El mensaje dice qué se usa, qué se declara y cómo se arregla:

```
the model uses the Storage 'acreditacion' in the StorageUse block 12, and no
StorageDef block declares it (the model declares: acreditacon). Declare
'acreditacion' with a StorageDef block, or use one of the declared names.
```

### `crates/genmsi-app/engine/src/session.rs`

`diagnostics_of` (el motor de validación de la app) ya no devuelve sólo lo que
dice el compilador: le suma los avisos del formato. Con eso el panel de
diagnósticos de la app (`DiagnosticsPanel.tsx`, que ya pinta
`diagnostics.diagnostics`) los muestra sin tocar el frontend, y el estado
pasa de `ok` a `diagnostics` cuando hay alguno. Un modelo que el motor
rechaza sigue mostrando su error, con los avisos del formato debajo.

## Lo que encontró la comprobación en los modelos de referencia

- **`Parcial 8-11-21__ParcialMikelModelos.gmdl`**: un `StorageUse` usa el
  almacén `acreditacion` y el `StorageDef` declara `acreditacon`. Es un typo
  del modelo, y es el caso exacto del que habla el encargo (el motor le
  pondría capacidad 1.000.000 al almacén `acreditacion`). El mismo modelo usa
  además cinco save values que ningún `InitiaSV` declara
  (`paradescongelar`, `descongeladas`, `CountPaciente`) y hace un `Tabulate`
  de la tabla `Tabla` cuando declara `Tabla1` y `vacunasfaltantes`.
- **`TP10__tp10.gmdl`**: un `Counter` usa el save value `CountEspera`, que
  ningún `InitiaSV` declara (el modelo declara otros cinco).

Ningún otro modelo de referencia usa un almacén, una tabla o una función sin
declarar (eso es lo que fija el test `the_reference_models_use_no_undeclared_storage`).

## Lo que NO se tocó, y por qué

- **`genmsi-core`**: no se toca nada. El motor sigue poniendo la capacidad
  1.000.000 al almacén sin declarar; esa decisión es de la otra sesión. Esta
  rama no cambia ni una línea del core.
- **El documento `genmsi.lint/1` del CLI y del MCP**: no se toca. Es el
  contrato del compilador (`program.diagnostics`), y el test del MCP exige
  `status == "ok"` en los 25 modelos de referencia: con `TP10` y
  `Parcial 8-11-21` avisos, ese estado pasaría a `diagnostics` y habría que
  cambiar el test. Si quieres que el lint también lo diga, es un cambio de
  contrato y hay que decidirlo aparte.
- **`validate()` (el cargador)**: no falla por esto, a propósito (ver la
  decisión 1).

## Verificación

- `cargo test --workspace`: en verde.
- `cargo clippy --workspace --all-targets -- -D warnings`: en verde.
- Test del formato: `crates/genmsi-format/tests/declarations.rs`.
- Test de la app: `crates/genmsi-app/engine/tests/declarations.rs`.

## Lo que falta decidir

1. **Cola, facility, cadena y grupo**: hoy no se avisan (decisión 3). Si el
   criterio es "todo lo que GPSS crea al usarlo también debe declararse",
   hay que añadir esas clases a `KINDS` y aceptar el ruido sobre los modelos
   de referencia.
2. **El idioma del mensaje**: está en inglés, como el resto de los mensajes
   del crate (`duplicate module id`, `invalid block`) y como los diagnósticos
   del core (`TABULATE x has no TABLE definition`), que la app muestra dentro
   una interfaz en español. Si lo prefieres en español, se cambia en un
   sitio (`UndeclaredEntity::message`).
3. **El mensaje en la app**: hoy es una línea larga en el panel de
   diagnósticos. Si quieres que la app lo muestre por bloque (con el nodo del
   bloque, que ya va en el texto), hay que tocar el frontend.
