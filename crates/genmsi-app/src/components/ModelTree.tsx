import { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Folder,
  FolderOpen,
  Layers,
  Component,
} from "lucide-react";
import type { ModelBlock, ModelBody } from "../types";

/**
 * Hierarchical tree of the model and its containers/submodules.
 *
 * Reflects the "Modelos y Módulos" hierarchy from the original GenMSI application.
 * Allows navigating between nested submodules and inspecting their contents.
 */
export default function ModelTree({
  model,
  activeContainerId,
  selectedBlockIds = [],
  onSelectContainer,
  onSelectBlock,
}: {
  model: ModelBody;
  activeContainerId: number;
  selectedBlockIds?: number[];
  onSelectContainer: (id: number) => void;
  onSelectBlock?: (id: number) => void;
}) {
  // Collect all containers (kind === "C" or id === 0).
  const containers = model.blocks.filter((b) => b.kind === "C" && b.id !== 0);

  // Group blocks by their owning container id.
  const childrenByContainer = new Map<number, ModelBlock[]>();
  for (const block of model.blocks) {
    if (block.id === 0) continue;
    const parentId = block.container;
    const list = childrenByContainer.get(parentId);
    if (list === undefined) {
      childrenByContainer.set(parentId, [block]);
    } else {
      list.push(block);
    }
  }

  // Set of expanded container ids (root and all containers expanded by default).
  const [expanded, setExpanded] = useState<Set<number>>(() => {
    const all = new Set<number>([0]);
    for (const c of containers) {
      all.add(c.id);
    }
    return all;
  });

  const toggleExpand = (id: number, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const modelTitle = model.title.trim() || model.path?.split("/").pop() || "Modelo";

  return (
    <div className="flex flex-col gap-1 text-xs select-none">
      <div className="mb-2 flex items-center justify-between border-b border-slate-200 pb-2">
        <span className="font-semibold text-slate-700">Modelos y Módulos</span>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => {
              const all = new Set<number>([0]);
              for (const c of containers) all.add(c.id);
              setExpanded(expanded.size > 1 ? new Set([0]) : all);
            }}
            className="text-[11px] text-indigo-600 hover:text-indigo-800"
          >
            {expanded.size > 1 ? "Colapsar todo" : "Expandir todo"}
          </button>
        </div>
      </div>

      {/* Root model node */}
      <div className="flex flex-col">
        <div
          onClick={() => onSelectContainer(0)}
          className={[
            "group flex items-center gap-1.5 rounded px-2 py-1.5 cursor-pointer transition",
            activeContainerId === 0
              ? "bg-indigo-50 font-semibold text-indigo-900 ring-1 ring-indigo-300"
              : "text-slate-800 hover:bg-slate-100",
          ].join(" ")}
        >
          <button
            type="button"
            onClick={(e) => toggleExpand(0, e)}
            className="p-0.5 text-slate-400 hover:text-slate-700"
            aria-label={expanded.has(0) ? "Colapsar" : "Expandir"}
          >
            {expanded.has(0) ? (
              <ChevronDown className="size-3.5" />
            ) : (
              <ChevronRight className="size-3.5" />
            )}
          </button>
          <Layers className="size-4 text-indigo-600 shrink-0" />
          <span className="truncate flex-1">{modelTitle}</span>
          <span className="text-[10px] text-slate-400 font-mono">
            {childrenByContainer.get(0)?.length ?? 0} ítems
          </span>
        </div>

        {/* Root children */}
        {expanded.has(0) && (
          <div className="ml-3 pl-2 border-l border-slate-200 flex flex-col gap-0.5 mt-0.5">
            {renderTreeChildren(
              0,
              childrenByContainer,
              activeContainerId,
              selectedBlockIds,
              expanded,
              toggleExpand,
              onSelectContainer,
              onSelectBlock,
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function renderTreeChildren(
  containerId: number,
  childrenByContainer: Map<number, ModelBlock[]>,
  activeContainerId: number,
  selectedBlockIds: number[],
  expanded: Set<number>,
  toggleExpand: (id: number, e: React.MouseEvent) => void,
  onSelectContainer: (id: number) => void,
  onSelectBlock?: (id: number) => void,
) {
  const children = childrenByContainer.get(containerId) ?? [];
  if (children.length === 0) {
    return (
      <span className="py-1 px-2 text-[11px] italic text-slate-400">
        (vacío)
      </span>
    );
  }

  // Separate subcontainers and atomic blocks
  const subcontainers = children.filter((b) => b.kind === "C");
  const activityBlocks = children.filter((b) => b.kind !== "C");

  return (
    <>
      {subcontainers.map((container) => {
        const isCurrentActive = activeContainerId === container.id;
        const isExpanded = expanded.has(container.id);
        const subchildren = childrenByContainer.get(container.id) ?? [];
        const displayName = container.caption.trim() || container.name.trim() || `Contenedor ${container.id}`;

        return (
          <div key={container.id} className="flex flex-col">
            <div
              onClick={() => onSelectContainer(container.id)}
              className={[
                "group flex items-center gap-1.5 rounded px-2 py-1 cursor-pointer transition",
                isCurrentActive
                  ? "bg-indigo-100 font-semibold text-indigo-900 ring-1 ring-indigo-400"
                  : "text-slate-700 hover:bg-slate-100",
              ].join(" ")}
              title={`Abrir submódulo ${displayName} en el lienzo`}
            >
              <button
                type="button"
                onClick={(e) => toggleExpand(container.id, e)}
                className="p-0.5 text-slate-400 hover:text-slate-700"
                aria-label={isExpanded ? "Colapsar" : "Expandir"}
              >
                {isExpanded ? (
                  <ChevronDown className="size-3" />
                ) : (
                  <ChevronRight className="size-3" />
                )}
              </button>
              {isExpanded ? (
                <FolderOpen className="size-3.5 text-amber-600 shrink-0" />
              ) : (
                <Folder className="size-3.5 text-amber-500 shrink-0" />
              )}
              <span className="truncate flex-1 font-medium">{displayName}</span>
              <span className="text-[10px] text-slate-400 font-mono">
                {subchildren.length}
              </span>
            </div>

            {isExpanded && (
              <div className="ml-3 pl-2 border-l border-slate-200 flex flex-col gap-0.5 mt-0.5">
                {renderTreeChildren(
                  container.id,
                  childrenByContainer,
                  activeContainerId,
                  selectedBlockIds,
                  expanded,
                  toggleExpand,
                  onSelectContainer,
                  onSelectBlock,
                )}
              </div>
            )}
          </div>
        );
      })}

      {activityBlocks.map((block) => {
        const isSelected = selectedBlockIds.includes(block.id);
        const blockName = block.caption.trim() || block.name.trim() || block.class;

        return (
          <div
            key={block.id}
            onClick={() => onSelectBlock?.(block.id)}
            className={[
              "flex items-center gap-1.5 rounded px-2 py-0.5 cursor-pointer text-[11px] transition",
              isSelected
                ? "bg-slate-200 font-medium text-slate-900 ring-1 ring-slate-400"
                : "text-slate-600 hover:bg-slate-50",
            ].join(" ")}
            title={`Seleccionar ${block.class} (${block.id})`}
          >
            <Component className="size-3 text-slate-400 shrink-0" />
            <span className="truncate flex-1">{blockName}</span>
            <span className="text-[9px] text-slate-400 font-mono">
              {block.class}
            </span>
          </div>
        );
      })}
    </>
  );
}
