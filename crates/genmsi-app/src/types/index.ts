import type { LintDocument as LintBody } from "./generated/LintDocument";
import type { JsonReport as ReportBody } from "./generated/JsonReport";
import type { ModelDocument as ModelBody } from "./generated/ModelDocument";
import type { TraceDocument as TraceBody } from "./generated/TraceDocument";

/**
 * A versioned document: the schema id plus the body fields.
 *
 * The body types are generated from the Rust types
 * (`crates/genmsi-app/scripts/generate-types.sh`); the envelope is the one
 * shape the engine's documents share.
 */
export type Schema<T> = T & { schema: string };

/** The `genmsi.lint/1` document. */
export type LintDocument = Schema<LintBody>;
/** The `genmsi.report/1` document. */
export type ReportDocument = Schema<ReportBody>;
/** The `genmsi.model/1` document. */
export type ModelDocument = Schema<ModelBody>;
/** The `genmsi.trace/1` document: the events of a run, in simulation order. */
export type TraceDocument = Schema<TraceBody>;

export type { LintBody, ReportBody, ModelBody, TraceBody };
export type { JsonBlock } from "./generated/JsonBlock";
export type { TraceEvent } from "./generated/TraceEvent";
export type { Entities } from "./generated/Entities";
export type { Facility } from "./generated/Facility";
export type { Queue } from "./generated/Queue";
export type { Storage } from "./generated/Storage";
export type { Chain } from "./generated/Chain";
export type { Table } from "./generated/Table";
export type {
  Diagnostics,
  SessionAnswer,
  SessionState,
} from "./protocol";

export type { ModelBlock } from "./generated/ModelBlock";
export type { ModelEdge } from "./generated/ModelEdge";
export type { ModelPort } from "./generated/ModelPort";
export type { ModelProperty } from "./generated/ModelProperty";
export type { ModelRect } from "./generated/ModelRect";
export type { CatalogueDocument } from "./generated/CatalogueDocument";
export type { CatalogueClass } from "./generated/CatalogueClass";
export type { CataloguePort } from "./generated/CataloguePort";
export type { CatalogueProperty } from "./generated/CatalogueProperty";
export type { EditCommand } from "./generated/EditCommand";
export type { Position } from "./generated/Position";
