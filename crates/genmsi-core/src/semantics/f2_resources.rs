//! The contracts of the family: the resources.
//!
//! The classes of this family are the ones that take and give back the
//! things a transaction needs: the facility it seizes and releases, the
//! storage units it enters and leaves, the gate that lets it through, and
//! the definitions that declare the storages.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Buffer",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the engine compiles the class into a `BUFFER` statement that does nothing at run time.",
            blocking: "It waits for nothing and holds nobody: the transaction leaves the block in the same instant it arrives.",
            resources: "It takes nothing and gives nothing back.",
            gotchas: "The catalogue's description of the class (it 'buffers the transactions that arrive and releases them one at a time') is not what the engine does: the engine's `Buffer` retains nobody. The original's report of `TP6__Clase 6_2_UltimoCliente` reads `39 26` in the row `A_Buffer` (26 of the 39 that arrived were still held there and only 13 had gone on to the `GateL1` row), while the engine's `Buffer` sends every arrival to the next block.",
        },
    ),
    (
        "Examine",
        Semantics {
            flow: "The transaction goes on by `Salida1` when it belongs to the group `Examine.Grupo` and by `Salida2` when it does not; the engine compiles it into an `EXAMINE` whose third operand is the destination of `Salida2`.",
            blocking: "It waits for nothing: it is a test of a membership, not a rendezvous, and the transaction goes on by one of the two outputs at once.",
            resources: "It takes nothing and gives nothing back: it only reads the group that the `JoinMe` and `JoinNumber` classes fill.",
            gotchas: "The class asks about the **transaction**; the number is the `ExamineNumber` class. The original refuses a model whose `EXAMINE` has no third operand ('Faltan parámetros'): the engine compiles it and says it out loud, so a model whose `Examine` has no `Salida2` gets a diagnostic that says the original would not run it.",
        },
    ),
    (
        "FacilityInt",
        Semantics {
            flow: "The class is five statements: the `Queue` of `FacilityInt.Queue` (when the property is given), the `PREEMPT`, the `DEPART`, the `ADVANCE` of `FacilityInt.Media` and the `RETURN`; the transaction ends at the module's `Salida1`.",
            blocking: "The transaction waits when the owner of the facility has a priority at least as high as its own (`ModoInterrupcion=Prioridad`, the default): it joins the pending chain of the facility, like a `Seize` that finds it busy. With `ModoInterrupcion=Incondicional` it always takes the facility.",
            resources: "It takes the facility and gives it back with the `RETURN`, after `Media` units; the owner it interrupts loses the `Advance` it was in and the time it had left is saved in the parameter `FacilityInt.Remanente`.",
            gotchas: "The module's `SalidaAlt` ('Propietaria') is not where the interrupted owner goes: the engine's `PREEMPT` statement carries only the module's `Salida1`, so the owner waits for the `RETURN` and then goes on with the time it had left (with `ModoArrebato=Remover` the owner is removed instead).",
        },
    ),
    (
        "FacilityUse",
        Semantics {
            flow: "The class is five statements: the `Queue` of `FacilityUse.Queue` (when the property is given), the `SEIZE`, the `DEPART`, the `ADVANCE` of `FacilityUse.Media` and the `RELEASE`; the transaction ends at the module's `Salida1`.",
            blocking: "The transaction waits at the `SEIZE` while the facility is busy, and it is in the `Queue` while it waits: the `DEPART` comes after the `SEIZE`, not before it.",
            resources: "It takes the facility at the `SEIZE`, holds it for `Media` units and gives it back at the `RELEASE`; the `DEPART` takes the transaction out of the queue only when the facility was taken.",
            gotchas: "The block is five rows in the report, not one. The row of the `SEIZE` counts the seizures and not the attempts, and the transactions that wait are counted in the row of the block they came from: the original's report of the probe `facilidadContencion` reads `6` in the `SEIZE` row and `120 114` in the `QUEUE` row, with the waiters in the `DELAY` column of the facility.",
        },
    ),
    (
        "GateF",
        Semantics {
            flow: "The transaction goes on by `Salida1` when the facility `GateF.Facility` is in the state the operator `GateF.op` asks for (`Libre`, the default, or `Ocupado`); the transaction that fails the test goes by `Salida2` when it is connected.",
            blocking: "When `Salida2` is not connected the transaction waits at the gate until the state of the facility changes; the release of that facility retries the waiters.",
            resources: "It takes nothing and gives nothing back: the gate only reads the state of the facility.",
            gotchas: "The row of the gate keeps its zeros (`0 0 0`): the attempt that fails the test is counted in the `RETRY` of the facility, and the transaction that waits is counted in the row of the block it came from.",
        },
    ),
    (
        "GateL",
        Semantics {
            flow: "The transaction goes on by `Salida1` when the logic switch `GateL.LogicSwich` is in the state the operator `GateL.op` asks for (`Set` or `Reset`; with no operator the engine opens the gate when the switch is reset); the transaction that fails the test goes by `Salida2` when it is connected.",
            blocking: "When `Salida2` is not connected the transaction waits at the gate until the switch changes; every `Logic` that changes a switch retries at once the waiters of every gate, and each of them tests the switch again.",
            resources: "It takes nothing and gives nothing back: the gate only reads the state of the switch.",
            gotchas: "The row of the gate keeps its zeros (`0 0 0`): the attempt that fails the test is counted in the `RETRY` of the switch, and the transaction that waits is counted in the row of the block it came from (the original's report of the bench `GateL` reads `58 58` in the row of the generator).",
        },
    ),
    (
        "GateS",
        Semantics {
            flow: "The transaction goes on by `Salida1` when the storage `GateS.Storage` is in the state the operator `GateS.op` asks for: the six operators are `Vacio`, `NoVacio`, `Lleno`, `NoLleno` (the default), `Disponible` and `NoDisponible`; the transaction that fails the test goes by `Salida2` when it is connected.",
            blocking: "When `Salida2` is not connected the transaction waits at the gate until the state of the storage changes; the `ENTER` and the `LEAVE` of that storage retry the waiters.",
            resources: "It takes nothing and gives nothing back: the gate only reads the state of the storage.",
            gotchas: "A storage that no `StorageDef` declares holds nothing and is never full: the gate opens for `NoLleno`, `Vacio` and `NoDisponible` and stays shut for `Lleno`, `NoVacio` and `Disponible`. The row of the gate keeps its zeros and the attempts that fail are counted in the `RETRY` of the storage.",
        },
    ),
    (
        "Release",
        Semantics {
            flow: "The transaction gives the facility `Release.Facility` back and goes on to `Salida1`.",
            blocking: "It waits for nothing; the release wakes the head of the pending chain of the facility, and that waiter retries the `SEIZE`.",
            resources: "It gives the facility back, and it is the only thing that does: ending the transaction does **not** release what it held, and the facility stays taken for the rest of the run.",
            gotchas: "The engine does not check that the transaction holds the facility: any transaction that reaches a `Release` frees it, even one that never seized it. A release wakes one waiter, not all of them.",
        },
    ),
    (
        "Seize",
        Semantics {
            flow: "The transaction takes the facility `Seize.Facility` and goes on to `Salida1`; when the facility is busy it waits at the block (the trace shows the `backup` to the block it came from).",
            blocking: "It waits while the facility is busy, in the pending chain of the facility (by priority, and then by arrival); what lets it go is the `Release` of the owner, which wakes the head of the chain.",
            resources: "It takes the facility and keeps it; the facility is given back only by an explicit `Release` — ending the transaction does **not** release it, and the facility stays taken.",
            gotchas: "The row of the `SEIZE` counts the seizures and not the attempts, and the transactions that wait are counted in the row of the block they came from: the original's report of the probe `facilidadContencion` reads `6` in the `SEIZE` row and `120 114` in the `QUEUE` row.",
        },
    ),
    (
        "StorageDef",
        Semantics {
            flow: "It is a definition: it produces no statement, it has no port and the transaction never reaches it.",
            blocking: "It blocks nothing.",
            resources: "It declares the storage `StorageDef.Storage` and its capacity in units (`StorageDef.Tamaño`); the storage exists from the start of the run.",
            gotchas: "The storage must be declared: a model that uses a storage nobody declares compiles, but the run stops when a transaction reaches it ('the storage 'S' is used but no StorageDef declares it'). A `StorageDef` without `Tamaño` declares a capacity of zero, so nothing ever fits and everybody waits at the `ENTER`.",
        },
    ),
    (
        "StorageUse",
        Semantics {
            flow: "The class is five statements: the `Queue` of `StorageUse.Queue` (when the property is given), the `ENTER` of `StorageUse.Cantidad` units, the `DEPART`, the `ADVANCE` of `StorageUse.Media` and the `LEAVE`; the transaction ends at the module's `Salida1`.",
            blocking: "The transaction waits at the `ENTER` while the storage is full, and it is in the `Queue` while it waits: the `DEPART` comes after the `ENTER`, not before it. The waiters are served by priority and, among the ones of the same priority, the last one that arrived goes in first (the opposite of a facility).",
            resources: "It takes `Cantidad` units of the storage `Storage` (one by default), holds them for `Media` units and gives them back at the `LEAVE`. The units are held, not consumed: a storage of capacity N admits N units at a time and the next transaction waits.",
            gotchas: "The storage must be declared by a `StorageDef`: the model compiles, but the run stops when a transaction reaches the storage. The row of the `ENTER` counts the entries that got in, and the transactions that wait are counted in the row of the block they came from.",
        },
    ),
];
