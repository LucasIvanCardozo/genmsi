# Sondas del lote 3 de la fase 7

Cuatro medidas que los bancos del lote no pueden dar, porque los bancos miden
el camino feliz y las sondas miden lo que el banco no toca. Cada sonda es un
modelo de los del arnés (`run_oracle.sh`), corrido contra el original y
contra nuestro motor.

| Sonda | Lo que pregunta | Modelo |
|---|---|---|
| 1 | ¿El `ASSIGN` pone el valor del parámetro? ¿El `SAVEVALUE` el suyo? | `AssignValor.gmdl` |
| 2 | ¿La llave lógica queda en 1? | `LogicSet.gmdl` |
| 3 | ¿La llave lógica con `Reset` queda en 0? | `LogicReset.gmdl` |
| 4 | ¿El `ALTER` altera al miembro del grupo? | `AlterMiembro.gmdl` |

Las cuatro están en este directorio; sus respuestas del original, en `oracle/`
y las nuestras, en `ours/`.

## Sonda 1 — el `ASSIGN` y el `SAVEVALUE`

**La pregunta.** El banco del `Assign` pone `Param,0` y no dice nada: no hay
ninguna cifra del reporte que diga si el valor llegó al parámetro de la
transacción. El `SaveValue` tampoco: su sección `SAVEVALUE` muestra el valor,
pero el del banco es 0, que es el mismo que el valor por defecto.

**El modelo.** Un `Assign` que pone `Param = 7`, un `SaveValue` que pone el
valor 5 y un `TEST` que compara `P:Param` con 7:

```
Generate ──► Assign (Param = 7) ──► SaveValue (1,5) ──► Test (E P:Param,7) ──┬─ A_Terminate
                                                                             └─ A_Terminate1
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` (la comparación se cumple) | `A_Terminate1` (no se cumple) | sección `SAVEVALUE` |
|---|---|---|---|
| original | **60** | 0 | `1` → **5** |
| nuestro | **57** | 0 | `1` → **5** |

**La regla.** El `ASSIGN` pone el valor en el parámetro de la transacción
activa ("Un bloque ASSIGN es usado para colocar o modificar un entero en un
parámetro de una transacción", manual del simulador), y el `SAVEVALUE` en el
valor guardado ("Un bloque SAVEVALUE es usado para asignar, incrementar, o
decrementar el valor de un valor guardado en la entidad"). Los dos motores lo
cumplen: la comparación con el parámetro se cumple en todas las transacciones
y el valor guardado es 5.

Lo que **no** se ve en esta sonda es la diferencia de la fila del `TEST`: la
del original escribe la referencia al parámetro como `P$Param` y la nuestra
como `P:Param`.

## Sonda 2 y 3 — la llave lógica

**La pregunta.** El banco del `Logic` pone la llave en 1 (`LOGIC S 1`), que es
el valor por defecto de una llave recién creada: el resultado no distingue "el
bloque la puso en 1" de "la llave ya valía 1". Y el manual dice que el
operador puede ser `S`, `R` **o `I`** (invertir), que nuestro motor no
conoce.

**El modelo.** Un `InitialLS` que pone la llave 1 en 1 y un `Logic` que la
pone en 1 (`LogicSet`) o la reinicia (`LogicReset`).

**La respuesta** (sección `LOGICSWITCH` del reporte del original):

| | `LOGIC S 1` (sonda 2) | `LOGIC R 1` (sonda 3) |
|---|---|---|
| original | valor **1** | valor **0** |
| nuestro | (no imprimimos la sección) | (no imprimimos la sección) |

**La regla.** El `LOGIC` configura la llave: con `S` la pone en "set" y con
`R` la pone en "reset" ("Si el operador lógico es S o R, la llave lógica
especificada por el operando A es puesta en el estado set o reset,
respectivamente", manual). Nuestro motor cumple la regla (el bloque `Logic`
del core la configura), pero **nuestro reporte no imprime la sección
`LOGICSWITCH`**, así que no se puede ver.

Con `LOGIC R 1` la llave queda en 0 **después** de que el `InitialLS` la
ponga en 1: o sea, el bloque `LOGIC` manda sobre el valor inicial.

## Sonda 4 — el `ALTER` y los miembros del grupo

**La pregunta.** El banco del `Alter` usa el grupo `ungrupo`, que nadie
define: no hay ningún miembro que alterar y la comparación del `ALTER` (el
parámetro `otro` contra el valor 1) no se cumple nunca. El banco no dice si
el `ALTER` altera al miembro del grupo, ni a cuál, ni qué pasa con la
transacción activa.

**El modelo.** Un `Assign` que pone `Param = 0`, un `JoinMe` que mete la
transacción en el grupo `UnGrupo`, el `ALTER` que altera el parámetro `Param`
de un miembro a 99 (comparando `Param` con 0), y un `TEST` que comprueba
`P:Param = 99`:

```
Generate ──► Assign (Param = 0) ──► JoinMe (UnGrupo) ──► Alter (E UnGrupo,1,Param,99,Param,0) ──┬─ Test (E P:Param,99) ──┬─ A_Terminate
                                                                                                  └─ A_Terminate2
                                                                                        (la salida de fallo del ALTER, "No alteró", va a A_Terminate1)
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` (el `TEST` se cumple) | `A_Terminate2` (no se cumple) | `A_Terminate1` (el `ALTER` no alteró a nadie) |
|---|---|---|---|
| original | **62** | 0 | 0 |
| nuestro | **62** | 0 | 0 |

**La regla.** El `ALTER` altera a los **miembros** del grupo, no a la
transacción activa: "Un bloque ALTER selecciona transacciones de un grupo y
altera uno de los atributos de dichas transacciones" (manual). Aquí la
transacción activa es el único miembro del grupo, así que es a ella a quien
altera, y su `Param` pasa a 99: por eso la comparación posterior se cumple en
las dos.

Lo que **no** coincide es la fila del `ALTER` del reporte (la del original
escribe `ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1`, con el destino de
la salida de fallo, y la nuestra `ALTER UnGrupo`), ni el listado del programa,
que no escribe ese séptimo operando.

**Un detalle del modelo, no del original.** En esta sonda los dos terminadores
de fallo se cruzan en el listado: el original numera sus terminadores en el
orden en que los crea y sale `A_Terminate2` antes que `A_Terminate1`. El
comparador lo ve como una diferencia de nombres, no de comportamiento: los
conteos de cada rama son los mismos en los dos motores.
