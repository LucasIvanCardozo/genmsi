# Fase 3b: genmsi-mcp — lo que el agente necesita para construir un modelo

Phase 3b session of genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12). Scope: `crates/genmsi-mcp/`, `crates/genmsi-core/` (additive public API only),
`crates/genmsi-format/` (read-only), `skills/`, `scripts/install-linux.sh`, `docs/`
(new documents; `DECISIONS.md` only to append an entry), `odd/tasks/`.

## Specs

- S1 (verbatim, user): "perfecto! vamos por paso 1 y 2 como tu recomendacion)" — the two
  steps are the skill (Paso 1) and the MCP changes (Paso 2: the catalogue that answers
  for many classes at once, the class that says which entities it declares and uses,
  and the server's instructions to a cold agent).
- S2 (verbatim, user): "la aplicacion debe poder ver que componentes tiene para diseniar
  el modelo y la creacion debe ser sencilla para el agente tanto para crear, anadir ver
  el modelo etc."
- S3 (verbatim, user): "no quiero que leas nada del workspace de genmsi, se supone que
  cree la aplicacion con mcp para que podamos trabajar con la aplicacion sin necesitar
  ver nada, si concideras que te falta info que el mcp no te lo da decime." The MCP
  alone has to be enough; the agent must not need the repository's source.
- S4 (D13, verbatim): "within one major version a change is additive only (new fields may
  appear; existing names, types and meanings stay), and a breaking change bumps the
  number."
- S5 (D14, verbatim): "`explain_model` has no CLI counterpart, so it publishes
  `genmsi.explain/1`: the engine's `program_json` plus the GPSS glossary of the classes
  the model uses."
- S6 (D11, verbatim): "Goal is minimum parity with the original, with better code quality
  — not a maximally professional system. Every tool addition must justify itself against
  that goal."
- S7 (ROADMAP Phase 3 exit criteria, verbatim): "an agent can build and simulate a model
  end to end via MCP."
- S8 (the measurement the user accepted with S1, verbatim from the coordinator's
  recommendation): "≤ 15 llamadas MCP y 0 lecturas del código fuente para un modelo de 9
  bloques" — a cold agent that only has the MCP and the skill must build the cafeteria
  model in that budget.
- S9 (the coordinator's decision, derived from S1-S3 and accepted by the user): the
  per-class `operands` table is **not** published. The order the engine renders is a
  property of 57 hand-written rows that drift the day the compiler changes; the same
  information reaches the agent from the model's own XML, where every property carries
  its name, and from the links that say how the model is wired.

## Tasks

- T1 (S1, S2, S4) `genmsi-mcp`: `catalogue` takes a list of classes (`classes: [...]`)
  next to the single `class`, and answers with `genmsi.classes/1`: the body of every
  class asked for, in one call. A class the catalogue does not have is an error that
  names the class. Tests: the batch returns every class asked for; the single-class
  answer stays byte-identical; an unknown class fails.
  Commit: `feat(mcp): the catalogue answers for a list of classes`.
- T2 (S2, S9) `genmsi-core` + `genmsi-mcp`: the class document says which entities the
  class **declares** (`EntityDef` properties: the storages, functions, tables, variables
  and logic switches a model has to declare) and which it **uses** (`EntityUse`
  properties: the facilities, queues and labels a model creates by using them). The
  two lists are read from the class's own properties, so they cannot drift from it.
  Tests: `StorageDef` declares a `Storage`; `FacilityUse` uses a `Facility`; the
  catalogue of every class carries both lists.
  Commit: `feat(mcp): every class says which entities it declares and uses`.
- T3 (S2, S3, S5) `genmsi-core` + `genmsi-mcp`: `explain_model` publishes the model's
  structure next to the compiled program: every block of the document with its class,
  its name, its properties **by name** and its ports, and the links that say how the
  blocks are wired. The agent reads the model as it is written instead of guessing.
  Tests: the links of a model with two blocks and one link are published; the
  properties of a block carry their names and values; the compiled program is still
  there.
  Commit: `feat(mcp): the model an agent reads says how it is wired`.
- T4 (S2, S3) `genmsi-mcp`: the server's `instructions` orient a cold agent in four
  lines: how two blocks are connected, that the class catalogue is the one source of
  the block schema, and that the skill carries the procedure.
  Commit: `feat(mcp): the server tells the agent how a model is wired`.
- T5 (S1, S2, S3) `skills/genmsi-modeling/SKILL.md`: the guide an agent follows to
  build a model: the grammar of the document, how two blocks are connected, the
  expression language, which entities a model declares and which it creates by using
  them, the runtime semantics that are not obvious, the workflow with the tools, one
  worked example and the traps. It never repeats the per-class schema: that is the
  catalogue's job.
  Commit: `docs(skill): the guide an agent follows to build a model`.
- T6 (S1) `scripts/install-linux.sh`: `--with-mcp` installs the skill into
  `~/.agents/skills/genmsi-modeling`, so the guide travels with the product.
  Commit: `feat(install): the installer ships the agent's guide`.
- T7 (S1, S4, S5) `docs/`: `docs/mcp.md` documents the new tools and the new fields;
  `docs/DECISIONS.md` appends the entry that says where each kind of knowledge lives
  (the catalogue holds the facts of a class, the explain document holds the structure
  of a model, the skill holds the procedure).
  Commit: `docs(mcp): the contract of the tools an agent uses`.
- T8 (S7, S8) measurement: a cold agent, with only the MCP and the skill, builds the
  cafeteria food-delivery model (nine blocks) and the count of tool calls and of
  repository reads is written in the Log of this document.
  Commit: none (the Log carries the result).

## Log

- L1 (verbatim, user, the request that opened this work): "bien quiero que veas lo que
  esta haciendo la sesion de "Estrategias para productos de software", porque? porque
  literal estoy probando el mcp y la app que creamos aqui, le dije que me creaara un
  modelo para x caso. Pero fijate que esta dando demasiadas vueltas y estoy seguro de
  que el problema es nuestro, ya sea el mcp o lo que sea. La aplicacion debe poder ver
  que componentes tiene para diseniar el modelo y la creacion debe ser sencilla para el
  agente tanto para crear, anadir ver el modelo etc. siento que eso no pasa actualmente.
  analiza y dame una opinion, y si ves necesario podes investigar en internet si hay
  algunos ejemplos de buenas practicas o cosaas asi para que mejoremos esto"
- L2 (evidence, the session under test): the session `01a16f17` (Estrategias para
  Productos de Software) spent 61 `codemode` batches, 21 `bash` calls and 4 reads to
  build a nine-block model. It asked the catalogue for 57 of the 66 classes, one class
  per call; it discovered the `<link>` grammar by reading the parser's error, the order
  of the operands by probing, the entity-declaration rules by reading
  `crates/genmsi-format/src/declaration.rs` (the user had forbidden it: S3), and the
  expression language by nine probes. The run cost about 182 `simulate`, 131 `lint` and
  119 `get_trace` calls, most of them discovering a fact instead of checking the model.
- L3 (decision, coordinator): the analysis recommended three layers — the facts of a
  class in the catalogue, the procedure in a skill, the interface in the tools — and
  the user chose "Paso 1 y 2", leaving the mutation tools (`add_block`, `connect`,
  `set_property`) out until the measurement of T8 justifies them. The reasoning: the
  cost of the cold session was information, not tools.
- L4 (decision, coordinator): the per-class `operands` table was dropped (S9). The
  engine renders the operands of a class in a fixed order (`report.rs::operand_parts`),
  and publishing it means a table of 57 rows written by hand next to the compiler that
  decides it. The model's own XML carries every property with its name, and T3
  publishes it, so the agent reads "Media" instead of guessing which position of
  "1,0,,," it is.
- L5 (evidence, the catalogue): the class list of 66 classes weighs 10,963 bytes and 15
  of them come back with an empty description. The class detail is one call per class.
  That is the N+1 the T1 tool removes.
- L6 (decision, coordinator): the skill is written in English, the language of the
  repository's technical documents and of the engine's glossary; the code comments of
  the shell installer follow the file's own Spanish.
