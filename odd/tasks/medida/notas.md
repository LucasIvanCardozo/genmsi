# Comedor de la facultad — modelo GenMSI

Pedido del usuario (verbatim): simular con GenMSI la entrega de comida del comedor: la
entrega arranca a las 12:00; hay **un cajero** (pide DNI, plata, da ticket) y **un responsable**
que calienta las bandejas (una por alumno) en el horno **de a tandas** y después las entrega a
los que ya tienen ticket; la fila para pagar **se forma antes de que abra el comedor**, con los
alumnos llegando ~20 min antes de las 12.

No hubo nadie a quien preguntarle: todas las suposiciones están abajo y son mías.

---

## 1. Suposiciones y números elegidos

| Parámetro | Valor | Por qué |
|---|---|---|
| Unidad de tiempo | **1 unidad = 5 segundos** | El reloj es discreto y los delays se truncan: con minutos el cobro (20 s) y la entrega (5 s) se volverían 0. Con 5 s todo queda con resolución. |
| `t = 0` | **11:30** | Así 12:00 es `t=360` y el horizonte 14:00 es `t=1800`, números manejables. |
| Llegada de alumnos | `Generate`: Media **6** (30 s), Modificador **3** (15 s), OffSet **120** (11:40), Cantidad **240** | "Los alumnos llegan 20 min antes de las 12" → OffSet = 120 unidades = 20 min antes de las 12. 240 alumnos = un comedor de facultad mediana; 1 cada 30 s ≈ 2/min. |
| Apertura del cajero | `t = 360` (12:00) | Lo pide el usuario. Se modela **reteniendo el recurso `Cajero`** desde t=0 (ver §3), no con un gate. |
| Cobro | `FacilityUse` Media **4** (20 s), Modificador **`FN:ExpoCobro`** | DNI + plata + ticket ≈ 20 s; distribución exponencial (la moda GPSS para un servicio). El truncamiento a unidades enteras deja el medio real en ~18 s. |
| Bandejas por tanda | **30** (capacidad del horno) | "de a tandas (según las que entren en el horno)". `StorageDef Horno` Tamaño=30. |
| Horneado | `Advance` Media **96** (8 min) | 30 bandejas por 8 min = 3,75/min, apenas por encima del cajero (~3,4/min): el horno alcanza, pero se forma cola de
bandejas. |
| Entrega de la bandeja | `FacilityUse Responsable` Media **1** (5 s) | El responsable entrega una bandeja cada 5 s; da para 12/min, no es el cuello de botella. |
| Horizonte | `Clock` Frecuencia **1800** (14:00) | El `Terminate` de los alumnos va con Unidades=0 (nunca corta la corrida), así que el reloj es el que cierra la simulación. |
| Ticket | `SaveValue TicketsEmitidos` + `Assign Ticket = X:TicketsEmitidos` | Numero de ticket correlativo 1..240. |

### Capacidad de las bandejas (decisión importante)
El depósito `BandejasListas` tiene Tamaño **30** (igual a la tanda). El alumno **no** puede tomar
una bandeja con `StorageUse` (ver §3, hallazgo 4): el `StorageUse` hace ENTER y necesita **lugar**
en el depósito, así que con el depósito lleno nadie puede sacar nada. Por eso el alumno toma la
bandeja con un `Leave(BandejasListas, 1)` (saca una bandeja del depósito), precedido por un
`GateS(BandejasListas, NoVacio)` que lo frena hasta que haya bandeja caliente.

---

## 2. Clases usadas y qué hace cada bloque en el escenario

**Definiciones (sin puertos):**

| id | clase | qué hace |
|---|---|---|
| 1 | `StorageDef` **Horno** (30) | Las bandejas que están horneándose. También marca "horno ocupado". |
| 2 | `StorageDef` **BandejasListas** (30) | Bandejas calientes ya sacadas del horno, esperando ser entregadas. |
| 3 | `StorageDef` **Pendientes** (100) | Alumnos que ya tienen ticket y todavía no reciben su bandeja. |
| 4 | `Table` **TablaEsperaCobro** | Histograma de la espera en la fila (Atributo `P:EsperaCobro`). |
| 5 | `Table` **TablaEsperaBandeja** | Histograma de la espera desde el ticket hasta la entrega. |
| 6-8 | `InitiaSV` ×3 | Tickets emitidos / alumnos servidos / lotes horneados (inician en 0). |
| 9 | `FunctionExpon` **ExpoCobro** | Servicio exponencial del cajero (usada como `Modificador`). |
| 10-12 | `Note` ×3 | Notas del escenario, de los tiempos y de la cocina (salen en el lienzo). |
| 13 | `Clock` | Horizonte: la corrida termina a los 14:00. |

**Flujo del alumno:**

| id | clase | qué hace |
|---|---|---|
| 14 | `Generate` **LlegadaAlumnos** | Los alumnos llegan desde las 11:40 (OffSet 120), 240 en total. |
| 15 | `Queue` **ColaFila** | **La fila que se forma antes de las 12**: el alumno entra a la fila al llegar. |
| 16 | `Mark` **MarcaLlegada** | Marca el instante de llegada (para medir la espera). |
| 17 | `FacilityUse` **CobroCajero** | **El cajero**: pide DNI, cobra, da ticket. Espera en `ColaCobro` si está ocupado. |
| 18 | `Depart` **SalirDeFila** | El alumno sale de la fila cuando lo atienden (la fila se acorta). |
| 19 | `Assign` **EsperaCobro** | `EsperaCobro = AC1 - P:MarcaLlegada`. |
| 20 | `Tabulate` | Tabula la espera en la fila. |
| 21 | `SaveValue` | Cuenta los tickets emitidos. |
| 22 | `Assign` | `Ticket = X:TicketsEmitidos` (número de ticket). |
| 23 | `Mark` **MarcaBandeja** | Arranca la espera de la bandeja. |
| 24 | `Enter` **Pendientes** | Queda registrado como pendiente de bandeja. |
| 25 | `GateS` **HornoLibre** | ¿Está el horno libre? (vacío) → Salida1: este alumno hace la tanda; Salida2: espera su bandeja. |
| 47 | `Test` **TamanoLote** | ¿Los pendientes ≤ 30? → True: tanda = pedidos; False: tanda = 30 (horno lleno). |
| 48 | `Assign` **LoteSegunPedidos** | `Lote = S:Pendientes`. |
| 49 | `Assign` **LoteHornoLleno** | `Lote = 30`. |
| 26 | `Enter` **CargarHorno** | Mete `P:Lote` bandejas al horno. |
| 27 | `Advance` **Hornear** | 8 minutos de horneado. |
| 28 | `FacilityUse` **SacarBandejas** | **El responsable** saca las bandejas del horno. |
| 29 | `Leave` **VaciarHorno** | Las bandejas salen del horno. |
| 30 | `Enter` **BandejasCalientes** | Las 30 (o las que son) bandejas calientes quedan listas para entregar. |
| 31 | `SaveValue` | Cuenta los lotes horneados. |
| 45 | `Queue` **ColaBandeja** | Cola de alumnos **con ticket** esperando su bandeja. |
| 44 | `GateS` **HayBandeja** | ¿Hay bandeja caliente? Si no, el alumno espera acá (se reintenta solo cuando cambia el depósito). |
| 32 | `Leave` **TomarBandeja** | El alumno toma su bandeja caliente. |
| 33 | `FacilityUse` **EntregaBandeja** | **El responsable** le entrega la bandeja (5 s). |
| 46 | `Depart` **SalirDeColaBandeja** | Sale de la cola de bandejas. |
| 34 | `Assign` **CalcularEsperaBandeja** | `EsperaBandeja = AC1 - P:MarcaBandeja`. |
| 35 | `Tabulate` | Tabula la espera de bandeja. |
| 36 | `Leave` **SalirDePendientes** | Ya no está pendiente. |
| 37 | `Counter` | Cuenta los alumnos servidos. |
| 38 | `Terminate` (Unidades **0**) | El alumno se va; **no** corta la corrida. |

**Flujo del turno del cajero (la apertura de las 12):**

| id | clase | qué hace |
|---|---|---|
| 39 | `Generate` **TurnoCajero** | Una transacción en t=0 (OffSet 0). |
| 40 | `Seize` **RetenerCajero** | El cajero retiene el recurso: **la caja está cerrada hasta las 12**. |
| 41 | `Advance` **HastaLas12** | Espera 360 unidades (30 min) → 12:00. |
| 42 | `Release` **AbrirCajero** | Abre la caja: el primero de la fila entra a ser atendido. |
| 43 | `Terminate` (Unidades 0) | Fin del turno. |

---

## 3. Suposiciones de modelado (y por qué)

1. **La fila se forma en `ColaFila` (cola explícita) y el cajero espera en `ColaCobro`
   (la cola interna del `FacilityUse`).** Son dos colas con el mismo contenido y por eso el
   report muestra las dos. `ColaFila` = "la fila física" (el alumno entra al llegar y sale
   cuando lo atienden); `ColaCobro` = "la espera del cajero" (con el tiempo de espera bien
   atribuido, que es lo que necesita el `Depart` de la fila).
2. **La caja cerrada hasta las 12 se modela reteniendo el recurso `Cajero`, no con un
   `GateL`.** Un `GateL` frenaría al alumno *dentro del bloque*, no en la cola: la fila se
   vería en el depósito de la cola pero **sin tiempo de espera atribuido**. Con el recurso
   retenido, la espera queda contabilizada en `ColaCobro` (`total_time` 14.297 = 5,3 min
   promedio, máximo 249 unidades = 20,7 min).
   *Consecuencia:* el `busy_time` del recurso `Cajero` (1.220) incluye las 360 unidades en que
   la caja estuvo cerrada: el tiempo de atención real es (1.220 − 360)/240 = 3,6 unidades ≈ 18 s.
3. **La tanda se arma por demanda**: el primer alumno con ticket que encuentra el horno libre
   es el que "carga" el horno, y la tanda es `min(30, S:Pendientes)`. Así no se cocinan bandejas
   al pedo y **no quedan bandejas sin usar al final** (con tanda fija de 30, la última tanda
   dejaba 4 bandejas sin repartir y 4 alumnos sin bandeja: se vio en una corrida anterior).
4. **El alumno que carga el horno es un alumno**: su transacción representa la tanda, hornea,
   y después se va él mismo a buscar su bandeja como todos. Es el recurso que dispara la cocina
   sin necesidad de una transacción aparte (un `Generate` de pedidos se acumularía en la cola).
5. **El `Terminate` de los alumnos va con Unidades=0.** `Unidades=1` decrementa el contador de
   arranque en 1 por cada transacción, así que con `start=1` **la primera** transacción atendida
   cortaba la corrida (me pasó: la corrida terminó en t=375 con 4 alumnos). El cierre lo hace el
   `Clock`.
6. **Se modela un solo cajero y un solo responsable** (el enunciado lo dice así). No hay
   ocupación de otros recursos ni limpieza extra.

---

## 4. Resultado de la última corrida (`simulate`, seed 42)

- `end_time` **1800.0** (= 14:00, lo corta el `Clock`), `end_reason` **terminated**
- **242 transacciones**, **17.863 eventos**
- **Entidades:** 5 colas, 2 recursos (`Cajero`, `Responsable`), 3 depósitos
  (`Horno`, `BandejasListas`, `Pendientes`), 3 save values, 2 tablas, 1 función.
- `TicketsEmitidos` **240**, `AlumnosServidos` **240**, `LotesHorneados` **12**.
- `ColaFila`: 240 ingresos, máximo **44** alumnos en la fila, espera total 15.157 (63 unidades
  = 5,3 min promedio; máximo 249 = 20,7 min → los que llegaron 11:40 esperaron hasta las 12).
- `ColaCobro`: máximo 43 esperando al cajero.
- `ColaBandeja`: máximo **64** alumnos esperando su bandeja; espera desde el ticket
  promedio **10 min**, máximo **40 min**.
- `Cajero`: 241 entradas, 1.220 de tiempo ocupado (360 de caja cerrada + 860 de atención).
- `Responsable`: 252 entradas, 252 unidades ocupadas (240 entregas + 12 descargas del horno).
- `Horno`: ocupación máxima 30 bandejas, 12 cargas. `BandejasListas`: máx. 30, quedan 2 al final.
- `Pendientes`: máximo 64, 0 al final (nadie quedó sin bandeja).

**Conclusión del modelo:** con estos números el cuello de botella es la **cocina**, no el cajero:
el alumno promedio pasa ~5 min en la fila y ~10 min esperando su bandeja, porque la comida sale
en tandas de 30 y los que llegan mientras el horno está ocupado esperan la tanda siguiente.

---

## 5. Lo que NO dice la guía y tuve que averiguar probando (sondas)

Todo esto sale de 3 modelos sonda (`lint` + `simulate` + `get_trace`) y de las corridas
intermedias del modelo real:

1. **`Queue` NO saca a nadie de la cola.** La guía dice "el bloque `Queue` deja la cola", pero en
   el trace la transacción "deja el bloque" y **queda en la cola**: el `content` de la cola siguió
   creciendo y nunca volvió a 0. Hay que poner un `Depart` (o usar la cola interna de
   `FacilityUse`/`StorageUse`, que sí hace `QUEUE, SEIZE, DEPART, ...`).
2. **`Generate.OffSet` corre el primer arribo.** La guía afirma que "un generador no llega en t=0,
   la primera transacción llega un intervalo después". Con `OffSet` presente, la primera
   transacción llega **exactamente en `OffSet`** (con `OffSet=0` llegó en t=0; con `OffSet=5` en
   t=5, con `Media=10`). Sin `OffSet` puesto, sí vale la regla de la guía.
3. **`Terminate.Unidades` descuenta del contador de arranque en cada transacción.** No es "el
   último", es "cada una": con `Unidades=1` y `start=1` la corrida termina con la primera
   transacción que pasa por ahí.
4. **`StorageUse` no toma un recurso del depósito: lo ocupa.** Es `QUEUE, ENTER, DEPART, ADVANCE,
   LEAVE`: el ENTER exige **lugar** en el depósito. Con el depósito lleno, el `StorageUse` no
   saca nada (espera), y con el depósito **vacío** le da una bandeja fantasma (no espera: entra
   igual y deja el depósito en -1/+1). Por eso el modelo toma la bandeja con `Leave` + `GateS`.
5. **`GateS` con `Salida2` conectada NO frena**: el que falla la prueba se va por `Salida2` en
   el acto; y si `Salida2` **no** está conectada, el que falla queda esperando en el bloque y se
   reintenta solo cuando cambia el estado (el `GateS(BandejasListas,NoVacio)` funciona así:
   los alumnos esperando bandeja se desbloquean cuando entra la tanda).
6. **El `Enter`/`Leave` de un depósito bloquea como una unidad con el resto de la cadena.** El
   trace muestra que un `Leave(Horno,30)` no se completa hasta que el `Enter(BandejasListas,30)`
   tiene lugar: es todo un bloque `backup`. Por eso el horno queda "ocupado" mientras espera
   lugar en el depósito de bandejas calientes.
7. **`Match` (rendezvous) hace que el par siga juntos y los dos consuman el recurso.** La sonda
   con dos `Match` + un `FacilityUse` dio `entries=2` y `busy_time=4` para un solo rendezvous:
   el alumno y la bandeja ocupan el recurso dos veces. Por eso la entrega se modeló con
   depósito + `FacilityUse` y no con `Match`.
8. **La `Table` tabula el valor del `Atributo` de la tabla, no el peso:** con
   `Atributo="P:Num"` y el parámetro en 42, la tabla registró 42 (`sum=42`). También funciona
   `Atributo="Num"` sin prefijo.
9. **`Assign.Valor` evalúa expresiones:** `AC1-P:MarcaLlegada` (la espera) y `X:TicketsEmitidos`
   (el número de ticket) funcionan; y `Cantidad="P:Lote"` funciona en `Enter`/`Leave`
   (el lote adaptativo).
10. **Las expresiones se compilan con `$`:** el GPSS generado muestra `S$Pendientes`, `P$Lote`,
    `FN$ExpoCobro` aunque en el archivo se escriba `S:Pendientes`, `P:Lote`, `FN:ExpoCobro`.
11. **`lint` informa un campo "blocks" que no es el conteo de bloques del modelo**: en el modelo
    final dio 50 y coincide con los 50 `<module>` del documento, pero en las sondas dio 22 y 27
    para modelos de 32 y 21 módulos. No sirve para contar bloques.
12. El `get_trace` numera los bloques con el **número de línea del GPSS − 1**, no con el
    `moduleid` del XML: hay que traducir para leer la traza.

## 6. Cosas que no pude / no quise hacer

- **No usé `Match`** para la entrega de la bandeja: el rendezvous es 1 a 1 y el par consume el
  recurso dos veces (hallazgo 7). Con depósito + `GateS` + `FacilityUse` la entrega de 1 a N sale
  limpia.
- **No usé `GateL`/`Logic`/`InitialLS`** para la apertura de las 12: frenan al alumno dentro del
  bloque y arruinan la métrica de la fila (hallazgo del punto 2 de §3). La lógica conmutada queda
  como clase no usada; el `Clock`+retención del recurso resuelven el cierre y la apertura.
- **El modelo no tiene horas de inicio/fin distintas por día ni días de la semana**: es un día
  representativo (martes, pico de mediodía).
- Si se quiere probar "otro día", lo natural es cambiar `OffSet` (llegada), `Media` del cajero y
  `Cantidad` de alumnos, y correr con distintas semillas.
