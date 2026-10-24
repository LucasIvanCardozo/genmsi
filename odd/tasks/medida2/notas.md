# Notas del modelo — entrega de comida en el comedor de la facultad

Archivo del modelo: `modelo.gmdl` (escrito con `save_model`).
Guía usada: `/home/lucas/.agents/skills/genmsi-modeling/SKILL.md` (único archivo leído fuera de este directorio).
Llamadas MCP: 24 (ver `mcp-calls.log`).

## 1. Supuestos que tomé (el usuario no respondió preguntas)

| Supuesto | Valor | Por qué |
|---|---|---|
| Unidad de tiempo | **segundos**, con reloj absoluto desde la medianoche | el modelo arranca a las 11:40 (42000) y el servicio a las 12:00 (43200); las 15:00 (54000) son el horizonte |
| Llegada de alumnos | `Media=25`, `Modificador=12` → interarribo uniforme(13, 37) s | "los alumnos van llegando 20 min antes de las 12" |
| La fila se forma | el primer alumno llega a las **11:40** (42000) y entra a la cola `ColaPago` en el acto | la línea existe antes de que abra el comedor |
| Apertura del cajero | **12:00** (43200), disparada por un `Generate` con `OffSet=43200` → `Logic` (Set) | el `Clock` no tiene puerto de salida, no puede disparar eventos |
| Servicio del cajero (DNI + plata + ticket) | `Media=15`, `Modificador=7` → uniforme(8, 22) s | cobro rápido, es el cuello de botella de la línea |
| Bandejas | **una por alumno con ticket**, creada con `Split` al emitir el ticket | garantiza el emparejamiento 1:1 alumno↔bandeja y evita bandejas huérfanas |
| Tanda del horno | **12 bandejas** = `Tamaño` del storage `Horno` | "de a tandas (según las que entren en el horno)"; la 13ª bandeja espera en el `Enter` |
| Calentado | `Media=180`, `Modificador=30` → uniforme(150, 210) s | el horno calienta la tanda en paralelo |
| Entrega | `Media=5`, `Modificador=2` s, por el responsable (facility `Responsable`, 1 persona) | el responsable saca la bandeja y se la da al alumno con ticket |
| Horizonte | `Clock` con `Frecuencia=54000` → la corrida termina a las 15:00 | fin del horario de almuerzo |

Otros supuestos: la línea es FIFO (el que llegó primero se atiende primero); el responsable es uno solo; el cajero es uno solo; no modelé el regreso de las bandejas (la bandeja se termina con la entrega).

## 2. El modelo, bloque por bloque

Flujo: `LlegadaAlumnos → MarcarLlegada → ColaPago → TomarCajero → EsperarAperturaCajero → TabularEsperaEnCola → CobrarYEmitirTicket → LiberarCajero → SalirDeColaPago → MarcarTicket → ContarTicketEmitido → CrearBandeja → (dos ramas)`.

| # | Clase | Caption | Qué hace en el escenario |
|---|---|---|---|
| 1 | Clock | RelojServicio | Horizonte de la corrida: cada 54000 s descuenta una unidad del START (termina 15:00) |
| 2 | Note | "Comedor: la fila…" | comentario |
| 3 | InitialLS | LlaveAperturaCajero | Declara la llave lógica `AperturaCajero` en estado **Reset** (el cajero abre a las 12:00) |
| 4 | StorageDef | Horno | Declara el depósito `Horno` con `Tamaño=12` (la tanda) |
| 5 | InitiaSV | InicialTicketsEmitidos | SaveValue `TicketsEmitidos` = 0 |
| 6 | InitiaSV | InicialComidasServidas | SaveValue `ComidasServidas` = 0 |
| 7 | Table | TablaEsperaEnCola | Tabla de 20 clases de 60 s; `Atributo = AC1-P:HoraLlegada` (tiempo esperado en la fila) |
| 8 | Generate | LlegadaAlumnos | Llegan los alumnos: media 25 s ±12, `OffSet=42000` (11:40), `Cantidad` vacío (infinito) |
| 9 | Generate | AperturaCajero | Un único evento a las 12:00 (`OffSet=43200`) que abre el cajero |
| 10 | Mark | MarcarLlegada | Guarda la hora de llegada del alumno en el parámetro `HoraLlegada` |
| 11 | Queue | ColaPago | El alumno entra a la fila de pago **desde que llega** (la línea crece antes de las 12) |
| 12 | GateL | EsperarAperturaCajero | No deja pasar al cajero hasta que la llave `AperturaCajero` esté en Set |
| 13 | Logic | AbrirCajero | Pone la llave en Set a las 12:00 |
| 14 | Seize | TomarCajero | Toma el recurso `Cajero` (el primero de la fila) |
| 15 | Advance | CobrarYEmitirTicket | 15 ±7 s: DNI + plata + entrega del ticket |
| 16 | Release | LiberarCajero | Devuelve el recurso `Cajero` |
| 17 | Depart | SalirDeColaPago | Saca al alumno servido de la cola `ColaPago` (la fila se acorta) |
| 18 | Assign | MarcarTicket | Marca el parámetro `TieneTicket=1` (el alumno ya tiene ticket) |
| 19 | SaveValue | ContarTicketEmitido | Suma 1 a `TicketsEmitidos` |
| 20 | Split | CrearBandeja | Duplica la transacción: **Salida1 = el alumno**, **Salida2 = la bandeja** (una bandeja por ticket) |
| 21 | Assign | MarcarBandeja | Marca la copia con `EsBandeja=1` |
| 22 | Enter | IngresarBandejaAlHorno | La bandeja entra al horno; si el horno está lleno (12) **espera en el bloque** (así se forman las tandas) |
| 23 | Advance | CalentarBandeja | 180 ±30 s de cocción |
| 24 | Leave | EgresarBandejaDelHorno | La bandeja sale del horno (libera el lugar de la tanda) |
| 25 | Seize | TomarResponsable | Toma el recurso `Responsable` (el único encargado de servir) |
| 26 | Match | MatchAlumno | Rendezvous con rótulo propio `Alumno` / conjugado `Bandeja`: el alumno con ticket espera su bandeja |
| 27 | Match | MatchBandeja | Rendezvous con rótulo propio `Bandeja` / conjugado `Alumno`: la bandeja caliente busca al alumno con ticket |
| 28 | Advance | EntregarBandejaAlAlumno | 5 ±2 s de entrega |
| 29 | Release | LiberarResponsable | Devuelve el recurso `Responsable` |
| 30 | SaveValue | ContarComidaServida | Suma 1 a `ComidasServidas` (comidas efectivamente servidas) |
| 31 | Tabulate | TabularEsperaEnCola | Anota el tiempo esperado en la fila (llegada → arranque del cobro) en `TablaEspera` |
| 32 | Terminate | FinAlumno | Descarta al alumno (no cierra la corrida) |
| 33 | Terminate | FinBandeja | Descarta la bandeja |
| 34 | Terminate | FinApertura | Descarta el evento de apertura |
| 35 | Note | "La tanda de 12 bandejas…" | comentario |

## 3. Resultado de la última `simulate` (seed 42, modelo guardado)

- `end_time` = **54000** (15:00), `end_reason` = `terminated`
- `transactions` = **953**, `events` = **13967**
- Cola `ColaPago`: `entries` 476, `max` **47** (la línea antes de las 12), `content` 1, `total_time` 77148 s
- Storage `Horno`: `capacity` 12, `max_use` **12** (la tanda llena), `entries` 475, `area` 85479
- Facilities: `Cajero` (476 usos, busy_time 8292 s), `Responsable` (469 usos)
- SaveValues: `TicketsEmitidos` = **475**, `ComidasServidas` = **469**
- Tabla `TablaEspera`: 476 muestras, máximo **1200 s** (el primero que esperó en la fila), media 147 s, 10 valores en overflow
- Lógica: `AperturaCajero` = true al final

## 4. Lo que la guía NO dijo y lo que las herramientas NO dijeron (hallazgos)

1. **`Generate.Cantidad = "0"` no significa "infinito"**: la guía y el catálogo dicen "0 means forever", pero con `"0"` el generador **no crea ninguna transacción** (0 entradas, la corrida queda vacía). El valor **vacío** es el que significa infinito. Lo descubrí porque la primera corrida del modelo completo dio `end_time 54000` con **2 transacciones**. Se corrigió y se comprobó con un generador de prueba (vacío → 38 llegadas en 1000 s; `"0"` → 0 llegadas).
2. **Los nombres de propiedad se validan en silencio**: `StorageDef` declara la capacidad en `Tamaño` (con ñ). Escribí `Tamano` y el `lint` respondió `status: ok` **sin diagnósticos**; el modelo compilaba pero el storage quedaba con `capacity: 0` y el `Enter` bloqueaba para siempre. El nombre tiene que ser literal y el error no se reporta.
3. **`GateL` libera a los que esperan en orden LIFO**: con la compuerta antes del `Seize`, al abrirse a las 12:00 la cola se vació en orden inverso (xact 46, 45, 44, …, 1): al último en llegar se lo atendía primero. Eso rompe el FIFO de la fila. Solución adoptada: la compuerta va **después** del `Seize` (el `Seize` sí es FIFO: verificado en el trace, orden de atención 1,2,3,…). Consecuencia: el primer alumno "ocupa" el cajero desde que llega hasta las 12:00, así que `Cajero.busy_time` incluye esa espera en silencio.
4. **`Match` necesita dos bloques con rótulos conjugados** (`RotuloPropio` de uno = `RotuloMatch` del otro); con un solo bloque no hay emparejamiento. En el trace el que llega primero queda en `backup` esperando al conjugado.
5. **El `Clock` no tiene puerto de salida**, así que no sirve para disparar eventos (p. ej. "abrir el cajero"): usé `Generate` con `OffSet` + `Logic`.
6. **El orden de los operandos compilados de `Generate` es (Media, Modificador, OffSet, Cantidad)**, no el orden de las propiedades del catálogo; el trace y el `report` lo muestran así.
7. **`Terminate` con `Unidades=0` descarta la transacción sin cerrar la corrida** (el `Clock` es el que termina); en el report ese `Terminate` aparece con operandos vacíos.
8. **El `Atributo` de la `Table` es una expresión evaluada en el momento del `Tabulate`**: verificado con exactitud en una prueba (llegada + `Advance` de 37 → valor tabularizado 37, clase 3 con ancho 10).
9. Numeración de bloques: el `report` tiene **una fila por sentencia** (no por bloque) y el número de bloque del `trace` es el número de línea del `report` **menos uno** (no el `moduleid`).
10. `Assign.Operador` acepta el `=` en texto plano (no hace falta el `%3D` del template).

### Sondas (probes) que hice y cuántas

| Sonda | Modelo | Llamadas | Qué reveló |
|---|---|---|---|
| P1 | miniatura del modelo completo (4 alumnos, horno de 2) | 3 (lint, simulate, simulate corregido) | Que el flujo GateL→Seize→Split→Enter/Leave→Match funciona; y el bug de `Tamaño` |
| P2 | 4 generadores: `Cantidad` vacía vs `"0"`, con y sin modificador, con/sin `OffSet` | 3 (lint, simulate, get_trace) | La semántica real de `Cantidad` y del `OffSet` (primera llegada exactamente en el offset) |
| P3 | `Mark` + `Advance` + `Tabulate` | 3 (lint, simulate, get_trace) | Que `AC1-P:HoraLlegada` es exactamente el tiempo transcurrido |

Sondas por clase: `Generate` 1 sonda (2 comparaciones), `StorageDef` 1 (P1), `Match` 1 (P1), `GateL` 1 (P1 + verificación en el modelo real), `Mark`/`Table` 1 (P3). Ninguna clase necesitó más de una sonda.

### Cosas que NO hice (y no son un problema del modelo)

- No leí el código del producto, ni `docs/`, ni `fixtures/`, ni los `odd/tasks/genmsi-*`, ni la corrida anterior `odd/tasks/medida/`. Todo lo que sé del formato viene de la guía y de las herramientas.
- No usé `Assemble`/`Gather` para la tanda: la última tanda parcial quedaría esperando para siempre y bloquearía el horno. La tanda se modela con la capacidad del `Horno` (12) + el `Enter` que espera.
- No usé `FacilityUse` para el cajero: su `JOIN` interno sumaría una segunda entrada por alumno en `ColaPago` y la cola no se vaciaría bien; usé `Queue` + `Seize` + `Advance` + `Release` + `Depart` explícitos.
