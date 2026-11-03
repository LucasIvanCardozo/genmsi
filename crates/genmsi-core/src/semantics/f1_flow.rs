//! The contracts of the family: the flow.
//!
//! The classes of this family are the ones that carry the transaction from
//! one block to the next: the generator that creates it, the queue it waits
//! in, the storage it enters, the branch it takes and the terminate that
//! ends it.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Advance",
        Semantics {
            flow: "The transaction stays in the block for `Advance.Media` units of simulated time and then goes on to `Salida1`. A delay of zero (or less) does not hold it at all: it goes on at once.",
            blocking: "It holds the transaction that arrives and nothing else; the delay is what lets it go. A numeric `Advance.Modificador` is a symmetric spread, a whole number drawn over `[Media-Modificador, Media+Modificador]`, and a function as the modifier multiplies the mean (`Media * f(u)`, truncated).",
            resources: "It takes nothing and gives nothing back: the transaction keeps the facility it seized and the storage units it entered while it waits.",
            gotchas: "The clock of the run is whole, so the transaction comes back at the truncated instant (a delay of 10.7 waits 10 units). `Advance.Media` is required: a block without it does not compile.",
        },
    ),
    (
        "Clock",
        Semantics {
            flow: "The `Clock` is the horizon of the run and not an event of the model: it has no port and sends no transaction anywhere. The engine compiles it into a generator that creates a transaction every `Clock.Frecuencia` units and a terminate that takes one unit of the run's counter for each of them.",
            blocking: "It blocks nothing: the transactions it creates go straight to its own terminate. What it decides is the end of the run: the run stops when the counter of the start reaches zero.",
            resources: "It takes one unit of the run's start counter per tick and nothing else: no queue, no facility, no storage. The tick that would come after the end of the run never happens.",
            gotchas: "The report writes two rows for a `Clock` (the `GENERATE` and the `TERMINATE` the engine compiles), and the model needs one: without a `Clock` and without a `Terminate` with a positive `Terminate.Unidades` the model does not run. A `Clock` with `Frecuencia` 60 ends a run with `start` 1 at time 60, and with `start` 2 at time 120.",
        },
    ),
    (
        "Depart",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the departure only takes the units out of the queue, and the transaction keeps its own way.",
            blocking: "It blocks nothing: the departure goes through even when the transaction is not in the queue, and it never waits for a facility or a storage.",
            resources: "It takes `Depart.Cantidad` units out of the queue `Depart.Queue` (at least one unit; a queue counts units, not transactions) and gives nothing back: the queue holds the transaction until a departure takes it out, and the time it waited is what the queue tables of that queue record.",
            gotchas: "The `Depart` of the engine touches the queue and nothing else: it does not release a facility, and the storage units of the transaction stay taken. A transaction that ends without a `Depart` stays in the queue, so the content of the queue only goes down when a `Depart` of that queue arrives.",
        },
    ),
    (
        "Enter",
        Semantics {
            flow: "The transaction takes `Enter.Cantidad` units of the storage `Enter.Storage` and goes on to `Salida1` as soon as the units fit.",
            blocking: "When the storage has no room the transaction waits in the chain of delay of the storage, served by priority (descending) and then by arrival, and it goes on when a `Leave` frees the units it needs.",
            resources: "It holds the units until a `Leave` gives them back, and a transaction that ends does not give them back: the storage stays occupied. The storage must exist: a `StorageDef` has to declare it, or the run refuses the model with `the storage 'X' is used but no StorageDef declares it`.",
            gotchas: "The capacity is in units (`StorageDef.Tamaño`), so a storage of `Tamaño` 1 with an `Enter.Cantidad` of 1 holds one transaction at a time and the others wait. The units of an `Enter` are at least one, so a `Cantidad` of 0 takes one unit; the row of the report holds the entries that got in, and the transaction that waits is counted in the block it came from.",
        },
    ),
    (
        "Generate",
        Semantics {
            flow: "The generator creates the transactions and sends each one to `Salida1`: the transaction is born in the generator and the flow goes on from there. A generator with no output creates the transactions and they end there.",
            blocking: "It blocks nothing: the generator does not wait for the transactions it creates, and a transaction that waits later (at a `Gate`, at an `Enter`) is still counted in the generator.",
            resources: "It takes nothing and gives nothing back. `Generate.Cantidad` limits the total (empty = forever, a number = that many, zero = none), `Generate.Media` is the mean of the intervals, `Generate.Modificador` the spread, `Generate.OffSet` the increment of the first arrival and `Generate.Prioridad` the priority of the transactions it creates.",
            gotchas: "The first arrival is not at time 0: it comes at `Generate.Media`, or at `Generate.OffSet` when the model sets one, and with a spread the first interval is drawn like the ones that follow. A `Generate.Cantidad` that is not a literal number does not limit the generator: the listing writes no limit and the generator creates transactions forever.",
        },
    ),
    (
        "Leave",
        Semantics {
            flow: "The transaction gives back `Leave.Cantidad` units of the storage `Leave.Storage` and goes on to `Salida1` at once.",
            blocking: "It never blocks: the units are given back even when the storage does not hold that many, and the occupancy goes down to zero, never below.",
            resources: "It returns the units to the storage and wakes one waiter of its chain of delay, which then takes its place in the `Enter`.",
            gotchas: "The units are given back by the transaction that leaves the storage, not by the one that ended: a transaction that ends while it holds units never gives them back. The storage must exist: a `StorageDef` has to declare it, or the run refuses the model.",
        },
    ),
    (
        "Priority",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the block only changes the priority of the transaction that arrives.",
            blocking: "It blocks nothing. The priority decides the order of the events of the same instant and the order of the chains of delay: a higher priority goes first and, among the ones of the same priority, the one that arrived first.",
            resources: "It takes nothing and gives nothing back: the priority is a property of the transaction, not a resource.",
            gotchas: "The value is rounded to a whole number, and the priority of the transaction is the one the generator gave it (`Generate.Prioridad`) until this block changes it.",
        },
    ),
    (
        "Queue",
        Semantics {
            flow: "The transaction joins the queue `Queue.Queue` and goes on to `Salida1` at once: the queue does not hold the transaction.",
            blocking: "It blocks nothing: the transaction continues its way and the queue only counts it. The wait of the transaction starts when it joins the queue and ends when a `Depart` takes it out.",
            resources: "It takes `Queue.Cantidad` units in the queue (at least one unit; a queue counts units, not transactions) and gives nothing back: only a `Depart` takes the transaction out of the queue.",
            gotchas: "A queue whose model has no `Depart` never empties, and a transaction that ends without a `Depart` stays in the queue: the content of the queue only goes down when a `Depart` of that queue arrives.",
        },
    ),
    (
        "Split",
        Semantics {
            flow: "The transaction goes on to `Salida1` (the original) and `Split.Cantidad` copies of it go to `Salida2` (the copies), born at the same instant and with the same priority.",
            blocking: "It blocks nothing: the copies are made at once and each one goes on its own way.",
            resources: "It takes nothing and gives nothing back: the copies carry a copy of the parameters of the original and share its assembly key, so an `Assemble` can gather them together with the original.",
            gotchas: "`Split.Cantidad` is the number of copies and not the number of transactions in all: `Cantidad` 2 makes two copies and the original continues (the models pair it with an `Assemble` of `1+Cantidad`). The `Param` property of the class, where the copies would be numbered, is ignored: the copies carry the values the original had.",
        },
    ),
    (
        "Terminate",
        Semantics {
            flow: "The transaction ends here: the class has no output port and the transaction goes on to no other block.",
            blocking: "It blocks nothing: the transaction that arrives ends at once. What the class decides is the end of the run: it takes `Terminate.Unidades` units from the counter of the start for every transaction that arrives, and the run ends when the counter reaches zero.",
            resources: "It gives nothing back: the facility the transaction had seized and the storage units it had taken stay taken, and the counter of the start is the only thing it touches. A `Terminate.Unidades` of 0 removes the transaction and never ends the run.",
            gotchas: "The class declares no default for `Unidades`: a `Terminate` without the operand never ends the run, and a model whose only way of ending is such a block does not run at all. A `Terminate.Unidades` that is not a literal number is 0.",
        },
    ),
    (
        "Transfer",
        Semantics {
            flow: "The transaction goes to `Salida1` with the probability `Transfer.Prob` and to `Salida2` with the rest; the draw is made at every arrival.",
            blocking: "It blocks nothing: the branch is taken at once.",
            resources: "It takes nothing and gives nothing back: the transaction keeps whatever it had on the way it takes.",
            gotchas: "`Transfer.Prob` is read in two ways: a literal with `%` is a fraction (`85%` is 0.85) and a bare number is per-mille (`850` is 0.85 and `100` is 10%). When `Salida2` is not connected the rest of the traffic goes to `Salida1` as well, and when `Salida1` is not connected the transaction that takes it ends there.",
        },
    ),
];
