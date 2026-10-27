# Geometria de puertos por clase (la verdad del editor original)

Fuente: `Genmsi_20230711.backup/gClasses.xml`, el catalogo de clases del
editor original. Cada clase declara su huella (`width`/`height`) y la
posicion de cada puerto en coordenadas de esa huella.

El modelo `.gmdl` **no** guarda la posicion de los puertos de un bloque de
actividad: la guarda el catalogo. Hoy `genmsi-format/src/catalogue.rs`
declara el `ioid`, el `name` y el `caption` de cada puerto, pero **no** su
ancla. El rediseño de las formas necesita esa ancla: sin ella, la app
reparte todos los puertos por igual (`portSpread`) y una `Test` pierde su
salida del vertice derecho.

Las coordenadas son de la huella de la clase. La tabla las normaliza:
`x%` e `y%` son la posicion dentro de la huella, para dibujar el ancla
sin importar el tamano real del bloque.

| Clase | Huella | Entradas (x%, y%) | Salidas (x%, y%) |
|---|---|---|---|
| `Advance` | 80x32 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Alter` | 100x50 | Entrada1 (40%, 0%) | Salida1 (10%, 100%), Salida2 (70%, 100%) |
| `Assemble` | 88x32 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Assign` | 100x45 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Buffer` | 88x32 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Clock` | 50x50 | — | — |
| `Counter` | 100x40 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Depart` | 97x32 | Entrada1 (36%, 0%) | Salida1 (36%, 100%) |
| `Enter` | 96x32 | Entrada1 (36%, 0%) | Salida1 (36%, 100%) |
| `Examine` | 80x48 | Entrada1 (50%, 0%) | Salida1 (25%, 83%), Salida2 (75%, 83%) |
| `ExamineNumber` | 80x48 | Entrada1 (50%, 0%) | Salida1 (25%, 83%), Salida2 (75%, 83%) |
| `FacilityInt` | 120x40 | Entrada1 (41%, 0%) | SalidaAlt (100%, 50%), Salida1 (41%, 100%) |
| `FacilityUse` | 100x40 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Function` | 100x100 | — | — |
| `FunctionDist` | 64x64 | — | — |
| `FunctionExpon` | 100x100 | — | — |
| `GateF` | 112x48 | Entrada1 (33%, 0%) | Salida1 (33%, 100%), Salida2 (0%, 50%) |
| `GateL` | 102x48 | Entrada1 (37%, 0%) | Salida1 (37%, 100%), Salida2 (0%, 50%) |
| `GateM` | 102x48 | Entrada1 (37%, 0%) | Salida1 (37%, 100%), Salida2 (0%, 50%) |
| `GateS` | 102x48 | Entrada1 (37%, 0%) | Salida1 (37%, 100%), Salida2 (0%, 50%) |
| `Gather` | 88x32 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Generate` | 120x63 | — | Salida1 (27%, 100%) |
| `InitiaSV` | 100x45 | — | — |
| `InitialLS` | 96x32 | — | — |
| `JoinMe` | 100x50 | Entrada1 (50%, 14%) | Salida1 (50%, 100%) |
| `JoinNumber` | 100x50 | Entrada1 (50%, 14%) | Salida1 (50%, 100%) |
| `Leave` | 96x32 | Entrada1 (36%, 0%) | Salida1 (36%, 100%) |
| `Link` | 98x50 | Entrada1 (35%, 20%) | Salida1 (35%, 80%) |
| `Logic` | 96x32 | Entrada1 (39%, 0%) | Salida1 (39%, 100%) |
| `Loop` | 96x32 | Entrada1 (59%, 0%) | Salida1 (59%, 100%), Salida2 (0%, 50%) |
| `MFunction` | 100x100 | — | — |
| `MFunctionDist` | 72x72 | — | — |
| `Mark` | 96x36 | Entrada1 (39%, 0%) | Salida1 (39%, 100%) |
| `Match` | 80x48 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Multifurcacion` | 80x20 | Entrada (50%, 0%) | Salida1 (0%, 100%), SalidaProb2 (25%, 100%), SalidaProb3 (50%, 100%), SalidaProb4 (75%, 100%), SalidaProb5 (100%, 100%) |
| `MultifurcacionP` | 80x20 | Entrada (50%, 0%) | Salida1 (0%, 100%), SalidaProb2 (25%, 100%), SalidaProb3 (50%, 100%), SalidaProb4 (75%, 100%), SalidaProb5 (100%, 100%) |
| `Note` | 80x80 | — | — |
| `Preempt` | 88x32 | Entrada1 (39%, 0%) | Salida1 (39%, 100%), SalidaAlt (100%, 100%) |
| `Priority` | 88x52 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `QTable` | 130x70 | — | — |
| `Queue` | 97x32 | Entrada1 (36%, 0%) | Salida1 (36%, 100%) |
| `Release` | 98x32 | Entrada1 (35%, 0%) | Salida1 (35%, 100%) |
| `RemoveMe` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 86%), Salida2 (100%, 20%) |
| `RemoveNumber` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 86%), Salida2 (100%, 20%) |
| `RemoveTrans` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 86%), Salida2 (100%, 20%) |
| `Return` | 88x32 | Entrada1 (39%, 0%) | Salida1 (39%, 100%) |
| `SaveValue` | 100x45 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Scan` | 80x50 | Entrada1 (50%, 0%) | Salida1 (25%, 80%), Salida2 (75%, 80%) |
| `Seize` | 98x32 | Entrada1 (35%, 0%) | Salida1 (35%, 100%) |
| `Select` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `SelectF` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%), Salida2 (100%, 50%) |
| `SelectL` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%), Salida2 (100%, 50%) |
| `SelectS` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%), Salida2 (100%, 50%) |
| `SelectSNA` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%), Salida2 (100%, 50%) |
| `Split` | 88x32 | Entrada1 (50%, 0%) | Salida1 (0%, 100%), Salida2 (100%, 100%) |
| `StorageDef` | 100x30 | — | — |
| `StorageUse` | 100x42 | Entrada1 (50%, 0%) | Salida1 (50%, 95%) |
| `Table` | 100x70 | — | — |
| `Tabulate` | 100x50 | Entrada1 (50%, 0%) | Salida1 (50%, 100%) |
| `Terminate` | 84x64 | Entrada1 (38%, 0%) | — |
| `Test` | 64x64 | Entrada1 (50%, 0%) | Salida1 (100%, 50%), Salida2 (0%, 50%) |
| `Transfer` | 64x64 | Entrada1 (50%, 0%) | Salida1 (100%, 50%), Salida2 (0%, 50%) |
| `Unlink` | 98x50 | Entrada1 (56%, 20%) | Salida1 (14%, 98%), SalidaAlt (98%, 52%), Salida2 (56%, 80%) |
| `Variable` | 120x32 | — | — |
| `VariableA` | 120x32 | — | — |
| `VariableN` | 120x40 | — | — |
