import { useMemo } from "react";
import { number } from "../format";
import { useLive } from "../monitor/live";
import { countAt } from "../monitor/series.ts";
import { closeFigures } from "../monitor/stats.ts";
import type { ReplayIndex, ReplayState } from "../monitor/replay";
import type { ModelBody, ReportDocument } from "../types";
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
import RunSummary from "./charts/RunSummary.tsx";
import StorageSeries from "./charts/StorageSeries.tsx";
import TableHistogram from "./charts/TableHistogram.tsx";
import TablesList from "./charts/TablesList.tsx";
import WaitDistribution from "./charts/WaitDistribution.tsx";

/**
 * The charts of the run.
 *
 * The section is one column that scrolls: the summary of the run first,
 * and then one group of charts per family of entities.
 *
 * The time series come from the replay (the state over the run, rebuilt by
 * walking the trace) and the histograms come from the report document (the
 * published frequencies). The charts read the documents; the only numbers
 * they compute are the ratios of the engine's numbers, and those say so.
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
  // The clock of the replay: the charts show the run as it is now, not the
  // run as it will be.
  const clock = useLive((live) => live.clock);
  // The figures of the entities at this instant, with the integrals that are
  // still open closed at the clock: the area of a queue is the area it has
  // covered so far.
  const closed = useMemo(
    () => (state === null ? null : closeFigures(state.figures, clock)),
    [state, clock],
  );
  // The time the numbers were measured over: the clock for the numbers of
  // the instant, the length of the run for the published ones. Each set of
  // numbers is divided by its own time.
  const elapsed = {
    live: clock - report.start_time,
    published: report.end_time - report.start_time,
  };

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
  const blocks = useMemo(
    () => blockRows(report, model, state),
    [report, model, state],
  );

  const hasTables =
    Object.keys(report.entities.tables).length +
      Object.keys(report.entities.qtables).length >
    0;

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto">
      <section className="rounded-lg border border-slate-200 bg-white p-4">
        <header className="mb-2 flex flex-wrap items-baseline justify-between gap-2">
          <h3 className="text-sm font-semibold text-slate-800">La corrida en números</h3>
          <span className="text-[11px] text-slate-500">
            Los números del instante, con el reloj de la reproducción
          </span>
        </header>
        <RunSummary
          report={report}
          state={state}
          clock={clock}
          generated={countAt(monitor.generateTimes, clock)}
        />
      </section>

      <ChartCard
        title="Los recursos que aprietan"
        subtitle="El recurso que más aprieta es el que más se acerca al 100 % o el que más cola carga."
        badge="derivado del reporte"
        badgeClass="shrink-0 rounded bg-amber-50 px-1.5 py-0.5 text-[10px] font-semibold text-amber-700"
        footnote="Utilización = tiempo ocupado / tiempo de la corrida. Contenido medio = área / tiempo de la corrida."
      >
        <ResourceRanking report={report} live={closed} elapsed={elapsed} />
      </ChartCard>

      <div className="grid gap-3 lg:grid-cols-2">
        <ChartCard
          title="Contenido de las colas a lo largo del tiempo"
          subtitle="Cada escalón es un instante en que la cola cambia: no es una muestra, es la historia exacta."
          badge="replay"
        >
          <QueueSeries monitor={monitor} queues={queues} clock={clock} />
        </ChartCard>

        <ChartCard
          title="Distribución de las esperas"
          subtitle="Cada barra son las transacciones que esperaron ese tiempo; la curva, su acumulada."
          badge="derivado de la traza"
          badgeClass="shrink-0 rounded bg-amber-50 px-1.5 py-0.5 text-[10px] font-semibold text-amber-700"
        >
          <WaitDistribution monitor={monitor} clock={clock} />
        </ChartCard>

        <ChartCard
          title="Indicadores por cola"
          subtitle="Los indicadores con * se derivan del reporte del motor: el área y el tiempo total, divididos por el tiempo de la corrida."
        >
          <QueueTable queues={queues} />
        </ChartCard>

        {facilities.length > 0 && (
          <ChartCard
            title="Ocupación de las facilities a lo largo del tiempo"
            subtitle="Cada tramo es un intervalo: el color es la facility ocupada, el gris la libre."
            badge="replay"
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
            subtitle="La utilización y el servicio medio se derivan del reporte del motor; las interrupciones y los reintentos los cuenta el motor."
          >
            <FacilityTable facilities={facilities} />
          </ChartCard>
        )}

        {storages.length > 0 && (
          <ChartCard
            title="Unidades en uso — almacén"
            subtitle="Las unidades ocupadas en cada instante y el límite del almacén: cuando la curva toca el límite, el modelo está bloqueado."
            badge="replay"
          >
            <StorageSeries monitor={monitor} storages={storages} clock={clock} />
          </ChartCard>
        )}

        {chains.length > 0 && (
          <ChartCard
            title="Tamaño de la cadena"
            subtitle="Cuántas transacciones hay encadenadas a la vez: la cadena crece cuando una entra y mengua cuando sale."
            badge="replay"
          >
            <ChainSeries monitor={monitor} chains={chains} clock={clock} />
          </ChartCard>
        )}

        {hasTables && (
          <ChartCard
            title="Histograma de la tabla"
            subtitle="La tabla del modelo: cada barra es un intervalo de valores y la curva su acumulada. La banda es la desviación."
            badge="reporte del motor"
          >
            <TableHistogram report={report} live={closed} />
          </ChartCard>
        )}

        {hasTables && (
          <ChartCard
            title="Las tablas del modelo"
            subtitle="Cada tabla que el modelo tabula, con lo que el motor publica de ella."
            badge="reporte del motor"
          >
            <TablesList report={report} live={closed} />
          </ChartCard>
        )}

        <ChartCard
          title="Transacciones por bloque"
          subtitle="Cuántas transacciones pasaron por cada bloque. La barra más larga es donde se acumula el trabajo."
          badge="reporte del motor"
          footnote="Un bloque del modelo puede ser varios del programa: la cifra es la de la fila que cuenta el bloque, para no contarla cinco veces."
        >
          <BlockRanking rows={blocks} />
        </ChartCard>

        <ChartCard
          title="Reintentos por bloque — la contención"
          subtitle="Un reintento es un intento que no pudo entrar: el bloque estaba ocupado o no había unidades. No es un error: es la señal del cuello de botella."
          badge="reporte del motor"
          badgeClass="shrink-0 rounded bg-amber-50 px-1.5 py-0.5 text-[10px] font-semibold text-amber-700"
        >
          <RetryRanking rows={blocks} />
        </ChartCard>
      </div>

      <p className="px-1 text-[11px] text-slate-400">
        Las cifras en violeta son las del instante que marca el reloj; las
        demás son las que el motor publicó para la corrida completa. El tiempo
        de todos los gráficos empieza en 0 y termina en{" "}
        {number(report.end_time - report.start_time)} s.
      </p>
    </div>
  );
}
