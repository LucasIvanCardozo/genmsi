import { useMemo } from "react";
import { useInstant, useLive } from "../monitor/live";
import { countAt } from "../monitor/series.ts";
import type { ReplayIndex, ReplayState } from "../monitor/replay";
import type { ModelBody, ReportDocument } from "../types";
import { decimals } from "../format.ts";
import BlockRanking from "./charts/BlockRanking.tsx";
import ChainSeries from "./charts/ChainSeries.tsx";
import ChartCard from "./charts/ChartCard.tsx";
import { blockRows } from "./charts/blocks.ts";
import {
  chainsOf,
  facilitiesOf,
  queuesOf,
  storagesOf,
} from "./charts/entities.ts";
import FacilityTable from "./charts/FacilityTable.tsx";
import FacilityTimeline from "./charts/FacilityTimeline.tsx";
import QueueSeries from "./charts/QueueSeries.tsx";
import QueueTable from "./charts/QueueTable.tsx";
import ResourceRanking from "./charts/ResourceRanking.tsx";
import RetryRanking from "./charts/RetryRanking.tsx";
import RunStatePanel from "./charts/RunStatePanel.tsx";
import RunSummary, { RUN_SUMMARY_FOOTNOTE } from "./charts/RunSummary.tsx";
import StorageSeries from "./charts/StorageSeries.tsx";
import TableHistogram from "./charts/TableHistogram.tsx";
import TablesList from "./charts/TablesList.tsx";
import WaitDistribution from "./charts/WaitDistribution.tsx";

/**
 * The charts of the run.
 *
 * The section is two columns: the cards of the charts on the left, and the
 * state of the run on the right, in a panel that does not scroll away with
 * the cards. In a narrow window the two stack and the panel goes first,
 * because it says what the run is before the charts say anything about it.
 *
 * The time series come from the replay (the state over the run, rebuilt by
 * walking the trace) and the histograms come from the report document (the
 * published frequencies). The charts read the documents; the only numbers
 * they compute are the ratios of the engine's numbers, and those say so.
 *
 * Every card says **where its numbers come from** —the badge, with the
 * state of the section— and **how they were made** —the foot, with the
 * formulas and the divisor. The divisor of every derived figure is the
 * clock of the instant: the time the reader is at, not the length of the
 * run.
 *
 * Every chart that has the time in its axis starts at **zero**: the axis
 * is the time of the run, from 0 to its end, and the vertical line marks
 * the instant the reader is at.
 */
export default function MonitorCharts({
  report,
  monitor,
  state,
  model,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The index of the replay. */
  monitor: ReplayIndex;
  /** The state of the replay at the instant the reader is at. */
  state: ReplayState | null;
  /** The model of the run, for the names of the blocks. */
  model: ModelBody | null;
}) {
  // The instant the numbers are of: the clock and the figures come from
  // the live layer, which publishes them together, so they are of the same
  // instant. The clock of the replay is the clock the reader is at, not
  // the run as it will be.
  const instant = useInstant(report.start_time, report.end_time, state);
  const clock = instant.clock;
  // The time the numbers were measured over: the clock for the numbers of
  // the instant, the length of the run for the published ones. Each set of
  // numbers is divided by its own time.
  const elapsed = {
    live: instant.live,
    published: instant.published,
  };
  // The figures of the entities at this instant, with the integrals that
  // are still open closed at the clock: the area of a queue is the area it
  // has covered so far.
  const closed = instant.figures;

  const queues = useMemo(
    () => queuesOf(report, closed, elapsed),
    [report, closed, elapsed.live, elapsed.published],
  );
  const facilities = useMemo(
    () => facilitiesOf(report, closed, elapsed),
    [report, closed, elapsed.live, elapsed.published],
  );
  const storages = useMemo(
    () => storagesOf(report, closed, elapsed),
    [report, closed, elapsed.live, elapsed.published],
  );
  const chains = useMemo(
    () => chainsOf(report, closed, elapsed),
    [report, closed, elapsed.live, elapsed.published],
  );
  // The counts of every block and the retries it could not be entered: they
  // come from the live layer too, so the ranking of the blocks is of the
  // instant the clock marks.
  const counts = useLive((live) => live.counts);
  const retries = useLive((live) => live.retries);
  const blocks = useMemo(
    () => blockRows(report, model, counts, retries),
    [report, model, counts, retries],
  );

  const hasTables =
    Object.keys(report.entities.tables).length +
      Object.keys(report.entities.qtables).length >
    0;

  // How many transactions were created up to the clock: the transactions
  // of the instant, against the total the engine published.
  const generated = countAt(monitor.generateTimes, clock);

  // The divisor of every derived figure: the clock of the instant, not
  // the length of the run.
  const until =
    instant.live > 0
      ? `tiempo hasta el reloj (${decimals(instant.live)} s)`
      : "tiempo hasta el reloj";

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto">
      <div className="grid items-start gap-3 lg:grid-cols-[minmax(0,1fr)_372px]">
        {/* The panel comes first in the order of the document: in a narrow
            window it stacks above the cards, and in a wide one the order
            puts it on the right, where it does not scroll away. */}
        <aside className="order-1 lg:order-2 lg:sticky lg:top-3">
          <RunStatePanel
            report={report}
            instant={instant}
            generated={generated}
          />
        </aside>

        <div className="order-2 flex min-w-0 flex-col gap-3 lg:order-1">
          <ChartCard
            title="La corrida en números"
            subtitle="Lo que el reloj ya no tiene que repetir: el reloj, el fin y la identidad de la corrida están en el panel lateral."
            badge="published"
            badgeLabel="mezcla de estados"
            footnote={RUN_SUMMARY_FOOTNOTE}
          >
            <RunSummary
              report={report}
              instant={instant}
              generated={generated}
              queues={queues}
            />
          </ChartCard>

          <ChartCard
            title="Los recursos que aprietan"
            subtitle="Cada barra tiene su escala: el tiempo que estuvo ocupada la facility, el máximo que alcanzó la cola o el de la capacidad del almacén."
            badge="derived"
            footnote={`Utilización * = tiempo ocupado ÷ ${until} · contenido medio * = área ÷ ${until} · utilización media * = área ÷ ${until} ÷ capacidad. Las familias no se comparan entre sí: cada una tiene su escala.`}
          >
            <ResourceRanking report={report} live={closed} elapsed={elapsed} />
          </ChartCard>

          <div className="grid gap-3 lg:grid-cols-2">
            <ChartCard
              title="Contenido de las colas a lo largo del tiempo"
              subtitle="Unidades en la cola, instante a instante: cada escalón es un cambio real, no una muestra."
              badge="instant"
              footnote={`Contenido medio * = área acumulada ÷ ${until}, no ÷ el tiempo de la corrida. A la derecha del reloj todavía no se sabe.`}
            >
              <QueueSeries monitor={monitor} queues={queues} clock={clock} />
            </ChartCard>

            <ChartCard
              title="Distribución de las esperas"
              subtitle="Cada barra son las transacciones que esperaron ese tiempo hasta el reloj; la curva, su acumulada."
              badge="instant"
              footnote="Media de las salidas * = suma de las esperas de las que salieron ÷ las que salieron. El ancho de los intervalos se elige una vez por cola, para que dos instantes de la misma cola sean comparables."
            >
              <WaitDistribution monitor={monitor} clock={clock} />
            </ChartCard>

            <ChartCard
              title="Indicadores por cola"
              subtitle="El contenido de este instante y las medias del motor, en las mismas cifras que muestra la serie de arriba."
              badge="instant"
              footnote={`Contenido medio * = área ÷ ${until} · espera media * = área ÷ entradas (la del motor) · espera cero * = entradas sin espera ÷ entradas · media de las salidas * = suma de las esperas de las que salieron ÷ las que salieron. Ahora y Máximo en unidades; Entradas en transacciones.`}
            >
              <QueueTable queues={queues} />
            </ChartCard>

            {facilities.length > 0 && (
              <ChartCard
                title="Ocupación de las facilities a lo largo del tiempo"
                subtitle="Cada tramo es un intervalo: el color es la facility ocupada, el gris el tiempo que estuvo libre."
                badge="instant"
                footnote={`Utilización = tiempo ocupado ÷ ${until}. El gris dice que estuvo libre hasta el reloj: a la derecha del reloj todavía no se sabe.`}
              >
                <FacilityTimeline
                  monitor={monitor}
                  facilities={facilities}
                  clock={clock}
                />
              </ChartCard>
            )}

            {facilities.length > 0 && (
              <ChartCard
                title="Indicadores por facility"
                subtitle="El estado de este instante y las medias del motor, con su fórmula en el pie."
                badge="published"
                badgeLabel="mezcla de estados"
                footnote={`Utilización = tiempo ocupado ÷ ${until} · servicio medio = tiempo ocupado ÷ entradas. Los reintentos de las compuertas los publica el motor para la corrida completa.`}
              >
                <FacilityTable facilities={facilities} />
              </ChartCard>
            )}

            {storages.length > 0 && (
              <ChartCard
                title="Unidades en uso — almacén"
                subtitle="Unidades ocupadas del almacén y su límite. Llenar el almacén no es lo mismo que estar bloqueado."
                badge="instant"
                footnote={`Utilización media * = área ÷ ${until} ÷ capacidad. Las esperas las cuenta la traza: son las veces que una transacción no pudo tomar unidades. Lo que dice si la corrida terminó bloqueada es el «esperando al final» del panel lateral.`}
              >
                <StorageSeries monitor={monitor} storages={storages} clock={clock} />
              </ChartCard>
            )}

            {chains.length > 0 && (
              <ChartCard
                title="Tamaño de la cadena"
                subtitle="Cuántas transacciones hay encadenadas a la vez: la cadena crece cuando una entra y mengua cuando sale."
                badge="instant"
                footnote={`Tiempo medio * = área ÷ entradas (la del motor) · tamaño medio * = área ÷ ${until}. El máximo es el de esta cadena, y las entradas son los enlaces hechos hasta el reloj.`}
              >
                <ChainSeries monitor={monitor} chains={chains} clock={clock} />
              </ChartCard>
            )}

            {hasTables && (
              <ChartCard
                title="Histograma de la tabla"
                subtitle="Cada barra es una clase de la tabla del modelo y la curva su acumulada; la banda, la media ± desviación."
                badge="published"
                badgeLabel="mezcla de estados"
                footnote="Media * = suma ÷ muestras · desviación * = √(E[x²] − E[x]²) · la media y la desviación de los desbordes las publica el motor, para la corrida completa. Las clases vacías del final se recortan."
              >
                <TableHistogram report={report} live={closed} />
              </ChartCard>
            )}

            {hasTables && (
              <ChartCard
                title="Las tablas del modelo"
                subtitle="Cada tabla que el modelo tabula, con lo que el instante sabe de ella."
                badge="instant"
                footnote="Media * = suma ÷ muestras · desviación * = √(E[x²] − E[x]²) · una tabla que el replay todavía no ha tabulado dice «todavía no»."
              >
                <TablesList report={report} live={closed} />
              </ChartCard>
            )}

            <ChartCard
              title="Transacciones por bloque"
              subtitle="Cuántas transacciones pasaron por cada bloque hasta el reloj, y cuántas cuenta el motor para la corrida completa."
              badge="published"
              badgeLabel="mezcla de estados"
              footnote="La primera cifra es la del instante y la segunda la que publicó el motor para la corrida completa. Un bloque del modelo puede ser varios del programa: la cifra es la de la fila que cuenta el bloque, para no contarla cinco veces. El generador crea las transacciones: su cifra cuenta las que creó."
            >
              <BlockRanking rows={blocks} />
            </ChartCard>

            <ChartCard
              title="Esperas por bloque — la contención"
              subtitle="Una espera es un intento que no pudo entrar: el bloque estaba ocupado o no había unidades. No es un error: es la señal del cuello de botella."
              badge="instant"
              footnote={`Las esperas las cuenta la traza del replay, no el motor, y están acumuladas ${until}.`}
            >
              <RetryRanking rows={blocks} />
            </ChartCard>
          </div>
        </div>
      </div>
    </div>
  );
}
