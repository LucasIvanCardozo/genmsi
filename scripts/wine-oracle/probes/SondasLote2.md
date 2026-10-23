# Sondas del lote 2 de la fase 7

Cuatro medidas que los bancos del lote no pueden dar, porque los bancos
miden el camino feliz y las sondas miden el camino que el banco no toca.
Cada sonda es un modelo de los del arnés (`run_oracle.sh`), corrido contra
el original y contra nuestro motor.

| Sonda | Lo que pregunta | Modelo |
|---|---|---|
| 1 | ¿Por qué rama va la `MultifurcacionP`? | `MultifurcacionPParam.gmdl` |
| 2 | ¿A dónde va la transacción que no cumple el `TEST`? | `TestFalso.gmdl` |
| 3 | ¿El contenido de una cola se acumula? | `QueueContent.gmdl` |
| 4 | ¿Cuánto retiene un `ADVANCE`? | `AdvanceDelay.gmdl` |

Las cuatro están en este directorio; sus respuestas del original, en
`/tmp/lote2-probes` (el arnés las deja ahí con el mismo nombre).

## Sonda 1 — la `MultifurcacionP` reparte por el valor del parámetro

**La pregunta.** El banco de la clase (`scripts/wine-oracle/benches/
MultifurcacionP.gmdl`) no dice nada del reparto: sin un parámetro, las
transacciones van todas a la primera rama, en los dos motores, y ese
resultado no distingue "reparte por el parámetro" de "siempre la primera".

**El modelo.** Un `Assign` antes del reparto, que le pone `Param = 3`:

```
Generate ──► Assign (Param = 3) ──► MultifurcacionP ──┬─ Salida1     ─► A_Terminate
                                                     ├─ SalidaProb2 ─► A_Terminate1
                                                     ├─ SalidaProb3 ─► A_Terminate2
                                                     ├─ SalidaProb4 ─► A_Terminate3
                                                     └─ SalidaProb5 ─► A_Terminate4
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` | `A_Terminate1` | `A_Terminate2` | `A_Terminate3` | `A_Terminate4` |
|---|---|---|---|---|---|
| original | 0 | 0 | **56** | 0 | 0 |
| nuestro | 0 | 0 | **60** | 0 | 0 |

**La regla.** La rama es el valor del parámetro, y las ramas se numeran desde
el uno: el `Salida1` es el valor 1, el `SalidaProb2` el 2, y así hasta el 5.
Los rótulos de las salidas de la clase lo dicen (`'[Parametro]=1'`,
`'[Parametro]=2'`, …) y el GPSS del original lo escribe
(`MultifurcacionGO FUNCTION P$Param,D5
1,A_Terminate/2,A_Terminate1/…`). Los dos motores cumplen la regla.

## Sonda 2 — el `TEST` con una comparación que falla

**La pregunta.** El banco de la clase compara `1` con `1`, así que la
comparación siempre se cumple: la salida de fallo ("Falso") no se usa nunca
y el banco no dice a dónde va la transacción que no la cumple.

**El modelo.** La comparación invertida (`1 = 2`) y las dos salidas
conectadas:

```
Generate ──► Test (E 1,2) ──┬─ Salida1 ("Verdadero") ─► A_Terminate
                            └─ Salida2 ("Falso")     ─► A_Terminate1
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` | `A_Terminate1` |
|---|---|---|
| original | 0 | **64** |
| nuestro | 0 | **60** |

**La regla.** La transacción que no cumple la comparación va por la salida de
fallo. Los dos motores la mandan igual. Lo que **no** coincide es el texto:
el GPSS del original escribe el destino como tercer operando
(`TEST E 1,2,A_Terminate1`) y el nuestro escribe `TEST E 1,2`. El reporte
nuestro sí lo escribe (por eso la diferencia no se ve en el banco de la
clase, donde la salida de fallo está sin conectar).

## Sonda 3 — el contenido de una cola sí se acumula

**La pregunta.** El banco de la clase encola y no saca nunca a nadie: el
contenido de la cola crece hasta el final de la corrida. Con ese banco se
puede ver cuánto crece, pero no se puede saber si el original cuenta el
contenido como lo contamos nosotros o si lo descuenta antes (al salir del
bloque `QUEUE`, por ejemplo).

**El modelo.** Un `Advance` de 1000 entre el `Queue` y el `Depart`, para que
las transacciones se queden paradas **fuera** de la cola antes de salir de
ella:

```
Generate ──► Queue ──► Advance (1000) ──► Depart ──► Terminate
Clock (3600) closes the run.
```

**La respuesta.**

| | `MAX` | `CONT.` | `ENTRIES` | `ENTRIES(0)` | `AVE.CONT.` | `AVE.TIME` |
|---|---|---|---|---|---|---|
| original | 20 | 15 | 60 | 0 | 14,507 | 870,400 |
| nuestro | 18 | 16 | 60 | 0 | 14,589 | 875,369 |

**La regla.** El contenido de la cola **se acumula** mientras la transacción
sigue viva: las dos cifras del `MAX` y del `CONT.` están dentro de la
tolerancia (35% en tiempos, 20% en conteos) y las de los promedios también.
El banco del `Queue` da el mismo veredicto por el otro lado (el `MAX` 61
contra 60 y el `CONT.` 61 contra 60).

## Sonda 4 — cuánto retiene un `ADVANCE`

**La pregunta.** El reporte no trae ninguna cifra del tiempo que una
transacción pasa en un `ADVANCE`, así que el banco de la clase (media 10,
desvío 5) sólo mide el tránsito.

**El modelo.** Un `ADVANCE` de 1000 sin desvío, con el reloj de 3600: las
transacciones que entran después de la hora 2600 no llegan al terminador
antes de que la corrida termine.

```
Generate ──► Advance (1000,0) ──► Terminate
Clock (3600) closes the run.
```

**La respuesta.**

| | entran en el `ADVANCE` | quedan dentro al final | llegan al `TERMINATE` |
|---|---|---|---|
| original | 57 | 16 | 41 |
| nuestro | 60 | 16 | 44 |

**La regla.** Los dos retienen lo mismo: **16 transacciones dentro del
`ADVANCE` al llegar la hora 3600**, y al terminador llegan las que entraron
antes de la hora 2600. La diferencia entre 41 y 44 no es la regla del
retardo: es que la corrida del original gerou 57 transacciones y la nuestra
60 (el generador sortea en cada corrida).

## Cómo repetirlas

```
# el original (unos 25 segundos por modelo)
for p in MultifurcacionPParam TestFalso QueueContent AdvanceDelay; do
  XDISPLAY=:96 bash scripts/wine-oracle/run_oracle.sh \
    scripts/wine-oracle/probes/$p.gmdl /tmp/lote2-probes
  wineserver -k
done

# el nuestro
cargo run --release -p genmsi-cli -- simulate scripts/wine-oracle/probes/<modelo>.gmdl
cargo run --release -p genmsi-cli -- gpss     scripts/wine-oracle/probes/<modelo>.gmdl
```
