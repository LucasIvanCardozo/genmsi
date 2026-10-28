import {
  fitName,
  fitTitle,
  groupByFamily,
  thumbnailOf,
  THUMBNAIL_BOX,
  type CatalogueEntry,
} from "../graph/catalogue";
import type { CatalogueDocument } from "../types";

/**
 * The classes of block the reader can create.
 *
 * The list lives in the lateral panel, in the tab **Bloques**: it is the
 * catalogue the model is built from, read at a glance. Every class shows
 * its silhouette — the same one the canvas draws — the name of the class
 * and what it does (the title of the catalogue), grouped by family, in two
 * columns.
 *
 * A class is created with a click — the block lands in the first free spot
 * of the model — or with a drag: the drag carries the name of the class in
 * the `application/genmsi-block` payload, which is what the picture reads
 * when the block is dropped on it.
 *
 * The list can be long, so it scrolls with the panel that holds it.
 */
export default function ClassList({
  catalogue,
  onAdd,
}: {
  /** The block classes the catalogue offers. */
  catalogue: CatalogueDocument | null;
  /** Add a block of a class. */
  onAdd: (className: string) => void;
}) {
  const classes = catalogue?.classes ?? [];

  if (classes.length === 0) {
    return (
      <p className="px-1 py-6 text-sm text-slate-500">El catálogo todavía no cargó.</p>
    );
  }

  const groups = groupByFamily(classes);

  return (
    <div className="flex flex-col gap-2">
      {groups.map((group) => (
        <section
          key={group.family ?? "otras"}
          className="flex flex-col gap-2 px-0 pb-1.5 pt-3"
        >
          <h3
            className="text-[11px] font-bold"
            style={{ color: group.colors?.acento ?? "#64748b" }}
          >
            {group.family ?? "Otras clases"}
          </h3>
          <ul className="grid grid-cols-2 gap-x-2 gap-y-1">
            {group.entries.map((entry) => (
              <li key={entry.className}>
                <ClassCard entry={entry} onAdd={onAdd} />
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

/**
 * One class of block: its silhouette, its name and what it does.
 *
 * The card is a button: clicking it creates the block. It is also
 * draggable, and the drag carries the name of the class, so the block lands
 * where the reader drops it.
 *
 * The silhouette is the thumbnail: the very path the canvas draws, scaled
 * with its proportion, without fill and without background — only the
 * outline, in the light tone of the family (S6).
 *
 * The thumbnail is big — 56×28 px, the box of the helper — so the class is
 * recognised by its shape before its name is read. The paddings of the list
 * are trimmed to pay for it: the text of the cell keeps the width it had.
 */
function ClassCard({
  entry,
  onAdd,
}: {
  /** The class the catalogue declares. */
  entry: CatalogueEntry;
  /** Add a block of the class. */
  onAdd: (className: string) => void;
}) {
  const thumbnail = thumbnailOf(entry.className);
  const name = fitName(entry.className);
  const title = fitTitle(entry.title);

  return (
    <button
      type="button"
      data-block-class={entry.className}
      title={`${entry.className} — ${entry.title}`}
      draggable
      onDragStart={(event) => {
        event.dataTransfer.setData("application/genmsi-block", entry.className);
        event.dataTransfer.effectAllowed = "copy";
      }}
      onClick={() => onAdd(entry.className)}
      className="flex w-full cursor-grab items-center gap-1.5 rounded-md border border-transparent px-1 py-[5px] text-left transition hover:border-indigo-200 hover:bg-indigo-50/50 active:cursor-grabbing"
    >
      {thumbnail !== null && (
        <svg
          width={THUMBNAIL_BOX.width}
          height={THUMBNAIL_BOX.height}
          viewBox={`0 0 ${THUMBNAIL_BOX.width} ${THUMBNAIL_BOX.height}`}
          aria-hidden="true"
          focusable="false"
          className="shrink-0"
        >
          <g
            transform={`translate(${thumbnail.offsetX} ${thumbnail.offsetY}) scale(${thumbnail.scale})`}
          >
            <path
              d={thumbnail.path}
              fill="none"
              stroke={thumbnail.stroke}
              strokeWidth={1.6}
              strokeLinejoin="round"
              vectorEffect="non-scaling-stroke"
            />
            {thumbnail.details.map((detail, index) => (
              <path
                key={index}
                d={detail}
                fill="none"
                stroke={thumbnail.stroke}
                strokeWidth={1.6}
                strokeLinejoin="round"
                vectorEffect="non-scaling-stroke"
              />
            ))}
          </g>
        </svg>
      )}
      <span className="min-w-0 flex-1">
        <span className="block truncate font-mono text-[11px] font-semibold text-slate-800">
          {name}
        </span>
        {entry.title !== "" && (
          <span className="block truncate text-[9px] leading-tight text-slate-500">
            {title}
          </span>
        )}
      </span>
    </button>
  );
}
