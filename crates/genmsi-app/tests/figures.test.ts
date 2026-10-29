//! The figure every block of the model shows.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  figureOf,
  figuresOf,
  propertiesByName,
  sameFigure,
  type BlockFigure,
  type FigureBlock,
  type FigureSource,
  type ReportFigureLike,
} from "../src/monitor/figures.ts";
import { useLive } from "../src/monitor/live.ts";
import type { Figures } from "../src/monitor/stats.ts";

/** A block of the model, as the figures read it. */
function block(id: number, className: string, properties: Record<string, string> = {}): FigureBlock {
  return { id, class: className, properties };
}

/** The figures of a run, with one entity of every kind. */
function runFigures(): Figures {
  return {
    queues: {
      ColaA: {
        content: 3,
        max: 5,
        entries: 7,
        zeroEntries: 0,
        totalTime: 12,
        area: 30,
        lastChange: 4,
      },
      ColaB: {
        content: 1,
        max: 2,
        entries: 3,
        zeroEntries: 1,
        totalTime: 2,
        area: 4,
        lastChange: 6,
      },
    },
    facilities: {
      Recurso: {
        busy: true,
        owner: 9,
        entries: 4,
        busyTime: 20,
        seizedAt: 30,
        retry: 0,
        inter: 0,
      },
    },
    storages: {
      Almacen: {
        inUse: 4,
        minUse: 0,
        maxUse: 6,
        entries: 9,
        area: 40,
        retry: 0,
        lastChange: 5,
      },
    },
    chains: {
      Cadena: {
        content: 2,
        max: 3,
        entries: 5,
        totalTime: 8,
        area: 10,
        lastChange: 6,
      },
    },
    tables: {
      Tabla: {
        samples: 11,
        maxValue: 9,
        sum: 30,
        sumSquares: 120,
        overflow: 0,
        frequencies: [3, 4, 4],
      },
    },
    logic: {
      Llave: true,
      LlaveApagada: false,
    },
    values: {
      Valor: 7,
      ValorCero: 0,
    },
  };
}

/** The report of a run, with the rows and the storages the figures read. */
function runReport(): ReportFigureLike {
  return {
    start_time: 0,
    blocks: [
      { line: 1, node: 1, class: "Generate", entries: 12, current: 0, counts_block: true },
      { line: 2, node: 2, class: "Terminate", entries: 10, current: 0, counts_block: true },
      { line: 3, node: 3, class: "FacilityUse", entries: 4, current: 0, counts_block: true },
      { line: 4, node: 3, class: "Seize", entries: 4, current: 0, counts_block: false },
    ],
    entities: {
      storages: { Almacen: { capacity: 20 } },
    },
  };
}

/** The source the figures are read from. */
function source(): FigureSource {
  return {
    figures: runFigures(),
    report: runReport(),
    clock: 50,
    counts: new Map([[9, { now: 2, ever: 30 }]]),
  };
}

/** The figure of a block, when it is the figure of an entity. */
function entity(figure: BlockFigure): BlockFigure & { type: "entity" } {
  assert.equal(figure.type, "entity", "the block shows the figure of an entity");
  if (figure.type !== "entity") {
    throw new Error("the figure is not the figure of an entity");
  }
  return figure;
}

/** The figure of a block, when it is the pair of always. */
function counts(figure: BlockFigure): BlockFigure & { type: "counts" } {
  assert.equal(figure.type, "counts", "the block shows the pair of always");
  if (figure.type !== "counts") {
    throw new Error("the figure is not the pair of always");
  }
  return figure;
}

test("a queue block shows the content of the queue it names", () => {
  const figure = entity(figureOf(block(1, "Queue", { Queue: "ColaA" }), source()));
  assert.equal(figure.kind, "queue");
  assert.equal(figure.entity, "ColaA");
  assert.equal(figure.text, "3");
  assert.equal(figure.label, "contenido de la cola");
});

test("the same block with another queue shows the other queue", () => {
  const figure = entity(figureOf(block(1, "Queue", { Queue: "ColaB" }), source()));
  assert.equal(figure.entity, "ColaB");
  assert.equal(figure.text, "1");
});

test("a block that leaves a queue shows the content of that queue", () => {
  const figure = entity(figureOf(block(1, "Depart", { Queue: "ColaA" }), source()));
  assert.equal(figure.kind, "queue");
  assert.equal(figure.text, "3");
});

test("a facility use shows the occupancy of the facility it names", () => {
  const figure = entity(figureOf(block(3, "FacilityUse", { Facility: "Recurso" }), source()));
  assert.equal(figure.kind, "facility");
  assert.equal(figure.entity, "Recurso");
  // The facility was held twenty seconds and it has been held since the
  // thirtieth: forty seconds of a run that has reached fifty is eighty per
  // cent, which is the arithmetic the engine uses for its utilization.
  assert.equal(figure.text, "80%");
  assert.equal(figure.label, "% de ocupación del recurso");
});

test("a storage use shows the units over the capacity", () => {
  const figure = entity(figureOf(block(4, "StorageUse", { Storage: "Almacen" }), source()));
  assert.equal(figure.kind, "storage");
  assert.equal(figure.entity, "Almacen");
  assert.equal(figure.text, "4/20");
  assert.equal(figure.label, "unidades / capacidad");
});

test("a tabulate shows the number of samples", () => {
  const figure = entity(figureOf(block(5, "Tabulate", { NombreTabla: "Tabla" }), source()));
  assert.equal(figure.kind, "table");
  assert.equal(figure.entity, "Tabla");
  assert.equal(figure.text, "11");
  assert.equal(figure.label, "nº de muestras");
});

test("a link shows the size of the chain", () => {
  const figure = entity(figureOf(block(6, "Link", { Cadena: "Cadena" }), source()));
  assert.equal(figure.kind, "chain");
  assert.equal(figure.entity, "Cadena");
  assert.equal(figure.text, "2");
  assert.equal(figure.label, "tamaño de la cadena");
});

test("a logic shows the value of the switch", () => {
  const figure = entity(figureOf(block(7, "Logic", { LogicSwich: "Llave" }), source()));
  assert.equal(figure.kind, "logic");
  assert.equal(figure.entity, "Llave");
  assert.equal(figure.text, "1");
  assert.equal(figure.label, "valor del interruptor");
});

test("a switch that is reset shows a zero, and not the pair of always", () => {
  const figure = entity(figureOf(block(7, "Logic", { LogicSwich: "LlaveApagada" }), source()));
  assert.equal(figure.text, "0");
});

test("a save value shows the value it keeps", () => {
  const figure = entity(figureOf(block(8, "SaveValue", { SaveValue: "Valor" }), source()));
  assert.equal(figure.kind, "value");
  assert.equal(figure.entity, "Valor");
  assert.equal(figure.text, "7");
  assert.equal(figure.label, "valor que guarda");
});

test("a value that is zero shows a zero, and not the pair of always", () => {
  const figure = entity(figureOf(block(8, "SaveValue", { SaveValue: "ValorCero" }), source()));
  assert.equal(figure.text, "0");
});

test("a generator shows the entries its row counted", () => {
  const figure = entity(figureOf(block(1, "Generate"), source()));
  assert.equal(figure.kind, "generated");
  assert.equal(figure.text, "12");
  assert.equal(figure.label, "flechas generadas");
});

test("a generator shows live generated counts when counts are available", () => {
  const liveSource: FigureSource = {
    ...source(),
    counts: new Map([[1, { now: 0, ever: 5 }]]),
  };
  const figure = entity(figureOf(block(1, "Generate"), liveSource));
  assert.equal(figure.kind, "generated");
  assert.equal(figure.text, "5");
  assert.equal(figure.label, "flechas generadas");
});

test("a terminate shows the transactions that finished", () => {
  const figure = entity(figureOf(block(2, "Terminate"), source()));
  assert.equal(figure.kind, "terminated");
  assert.equal(figure.text, "10");
  assert.equal(figure.label, "transacciones terminadas");
});

test("a terminate shows live terminated counts when counts are available", () => {
  const liveSource: FigureSource = {
    ...source(),
    counts: new Map([[2, { now: 0, ever: 3 }]]),
  };
  const figure = entity(figureOf(block(2, "Terminate"), liveSource));
  assert.equal(figure.kind, "terminated");
  assert.equal(figure.text, "3");
  assert.equal(figure.label, "transacciones terminadas");
});

test("a select of a facility shows the pair of always", () => {
  // The class chooses the facility (the first free one) and does not
  // declare it: the figure cannot be resolved, and no facility figure is
  // invented.
  const figure = counts(figureOf(block(9, "SelectF", { Param: "param" }), source()));
  assert.equal(figure.now, 2);
  assert.equal(figure.ever, 30);
  assert.equal(figure.text, "2/30");
  assert.equal(figure.label, "dentro / pasadas");
});

test("a block whose entity is a parameter shows the pair of always", () => {
  // The engine resolves the name of the entity when the transaction
  // arrives: the model cannot say it, so the block does not show a figure
  // of an entity it cannot name.
  for (const name of ["*caja", "[QCajas]"]) {
    const figure = counts(figureOf(block(9, "Queue", { Queue: name }), source()));
    assert.equal(figure.text, "2/30", `the queue named ${name}`);
  }
});

test("a block of a class that operates on no entity shows the pair of always", () => {
  const figure = counts(figureOf(block(9, "Advance"), source()));
  assert.equal(figure.text, "2/30");
});

test("the pair of always is the pair the block shows today", () => {
  const figure = counts(figureOf(block(9, "Advance"), source()));
  assert.equal(figure.now, 2);
  assert.equal(figure.ever, 30);
  assert.equal(figure.text, "2/30");
});

test("a block the run has not reached shows no figure at all", () => {
  const figure = counts(figureOf(block(11, "Advance"), source()));
  assert.equal(figure.text, "");
  assert.equal(figure.now, 0);
  assert.equal(figure.ever, 0);
});

test("an entity the run has not touched shows the pair of always", () => {
  // The queue is not in the figures: the engine has not published it, so
  // the block does not draw a number that is not the run's.
  const figure = counts(figureOf(block(9, "Queue", { Queue: "ColaNuncaUsada" }), source()));
  assert.equal(figure.text, "2/30");
});

test("a storage without a published capacity shows the pair of always", () => {
  const figures = runFigures();
  const report = runReport();
  report.entities.storages = {};
  const figure = counts(
    figureOf(block(4, "StorageUse", { Storage: "Almacen" }), {
      figures,
      report,
      clock: 50,
      counts: new Map([[4, { now: 2, ever: 30 }]]),
    }),
  );
  assert.equal(figure.text, "2/30");
});

test("the figures of every block are read from the same source", () => {
  const blocks = [
    block(1, "Queue", { Queue: "ColaA" }),
    block(2, "Advance"),
    block(3, "FacilityUse", { Facility: "Recurso" }),
  ];
  const figures = figuresOf(blocks, source());
  assert.equal(figures.size, 3);
  assert.equal(figures.get(1)?.type, "entity");
  assert.equal(figures.get(2)?.type, "counts");
  assert.equal(figures.get(3)?.type, "entity");
});

test("two figures that say the same thing are the same figure", () => {
  const one = figureOf(block(1, "Queue", { Queue: "ColaA" }), source());
  const other = figureOf(block(1, "Queue", { Queue: "ColaA" }), source());
  assert.equal(sameFigure(one, other), true);
  const moved = figureOf(block(1, "Queue", { Queue: "ColaB" }), source());
  assert.equal(sameFigure(one, moved), false);
  const pair = figureOf(block(1, "Advance"), source());
  assert.equal(sameFigure(one, pair), false);
});

test("the properties of a block are read by name", () => {
  const properties = propertiesByName([
    { name: "Queue", value: "ColaA" },
    { name: "Cantidad", value: "2" },
  ]);
  assert.deepEqual(properties, { Queue: "ColaA", Cantidad: "2" });
});

test("the live store carries the figures and keeps what did not change", () => {
  useLive.getState().reset();
  const before = runFigures();
  useLive.getState().publish({
    clock: 1,
    counts: new Map(),
    balls: new Map(),
    figures: before,
    blockFigures: new Map([[1, figureOf(block(1, "Queue", { Queue: "ColaA" }), source())]]),
  });
  const held = useLive.getState();
  assert.equal(held.figures, before);
  assert.equal(held.blockFigures.get(1)?.type, "entity");

  // The second publication carries the same figures: nothing moved.
  useLive.getState().publish({
    clock: 2,
    counts: new Map(),
    balls: new Map(),
    figures: before,
    blockFigures: new Map([[1, figureOf(block(1, "Queue", { Queue: "ColaA" }), source())]]),
  });
  const after = useLive.getState();
  assert.equal(after.figures, before, "the figures kept their identity");
  assert.equal(
    after.blockFigures.get(1),
    held.blockFigures.get(1),
    "the figure that did not change kept its identity",
  );
});
