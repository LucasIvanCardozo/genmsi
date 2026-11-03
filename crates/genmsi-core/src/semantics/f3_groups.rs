//! The contracts of the family: the groups and the chains.
//!
//! The classes of this family are the ones that hold transactions together:
//! the groups a transaction joins and leaves, the assembly sets that wait for
//! each other, and the user chains a transaction is linked to and unlinked
//! from.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Assemble",
        Semantics {
            flow: "The transaction waits at the block until `Assemble.Cantidad` transactions of its assembly set have arrived; the first one of the set goes on to `Salida1` and the ones that arrived later are destroyed and leave the run.",
            blocking: "It holds the transaction that arrives first in its set until the rest of the set shows up; the transactions of the other sets wait in the same block, and the engine tells the sets apart by their assembly key.",
            resources: "It takes nothing and gives nothing back: the destroyed transactions leave the simulation and the one that continues keeps the facility it seized and the storage units it entered.",
            gotchas: "The set is the assembly set, and only a `Split` creates one: two transactions that did not come from the same `Split` never complete a set and wait there forever. The batch is not released whole: the `Assemble` destroys the transactions that arrive later and lets the first one continue, and that is what the original does too (its report of `Parcial 6-11-23` reads 387 entries in the row `A_Assemble` and 20 in the row `A_Terminate`: only the transaction that opens each set goes on). The class that releases the whole set is `Gather`.",
        },
    ),
    (
        "Gather",
        Semantics {
            flow: "The transaction waits at the block until `Gather.Cantidad` transactions of its assembly set have arrived, and then the whole set goes on to `Salida1`, one transaction after the other.",
            blocking: "It holds the transaction that arrives first in its set until the rest of the set shows up; the transactions of the other sets wait in the same block, and the engine tells the sets apart by their assembly key.",
            resources: "It takes nothing and gives nothing back; every transaction of the set keeps the facility it seized and the storage units it entered.",
            gotchas: "The set is the assembly set, and only a `Split` creates one: two transactions that did not come from the same `Split` never complete a set and wait there forever. `Gather` is the class that releases the batch whole (the original's report of `Parcial 6-11-23` reads 367 entries in the row `A_Gather` and 367 in the row `GateL1` that follows it); `Assemble` destroys the transactions that arrive later and lets the first one continue.",
        },
    ),
    (
        "JoinMe",
        Semantics {
            flow: "The transaction joins the group `JoinMe.Grupo` and goes on to `Salida1` at once.",
            blocking: "It blocks nothing: the transaction is a member of the group from that instant and waits for nothing.",
            resources: "It takes nothing and gives nothing back; what it takes is a place in the group, and the group is created by use (no class declares it). The report counts in the group only the transactions that are still in the run.",
            gotchas: "It is the group of transactions: the `JoinNumber` class is the one that fills a numeric group. A transaction that ends is not taken out of the group: the engine keeps it in the list of the group and the report leaves it out of the size.",
        },
    ),
    (
        "JoinNumber",
        Semantics {
            flow: "The transaction adds the value of `JoinNumber.Numero` to the numeric group `JoinNumber.Grupo` and goes on to `Salida1` at once.",
            blocking: "It blocks nothing: the number is added and the transaction goes on, whatever the group holds.",
            resources: "It takes nothing and gives nothing back; what it adds is the number, and the group is created by use. The report counts in a numeric group the different numbers it holds, not the number of joins.",
            gotchas: "It is the numeric group: the `JoinMe` class fills the group of transactions. The one-line description the catalogue publishes says it waits until the group holds `Numero` transactions; the engine branches at once and never waits. The number is added as it is, with its repetitions, so a group joined a hundred times with the same number has one member.",
        },
    ),
    (
        "Link",
        Semantics {
            flow: "The transaction joins the user chain `Link.Cadena` and waits there; the first transaction that reaches the chain (the one that finds its link indicator off) goes on to `Salida1` instead of joining it, when that output is connected.",
            blocking: "It holds the transaction in the chain until an `Unlink` of the same chain releases it; the chain does not let go of anybody by itself.",
            resources: "It takes a place in the chain and gives it back when the transaction is unlinked; the chain is created by use (no class declares it) and the engine keeps its size, its maximum, its number of entries and the integral of its size over the time.",
            gotchas: "The chain is named by the `Link.Cadena` property and it is not a queue: it is a user chain, and the table it has in the report is `USER_CHAIN`. The order is `FIFO` or `LIFO` (`Link.Modo`); the `Link.Parametro` property, and the order by parameter that the original admits, are not implemented. The link indicator is set by the first `Link` and the engine never clears it, so every transaction that comes later waits in the chain even when the chain is empty again.",
        },
    ),
    (
        "Match",
        Semantics {
            flow: "The engine compiles the class into two statements: a `TRANSFER` that leads to the `MATCH`, and the `MATCH` itself, which carries the label `Match.RotuloPropio`. The transaction waits at the `MATCH` until a partner waits at the conjugate block, and then both go on, each one by its own `Salida1`.",
            blocking: "It holds the transaction at the block until the conjugate block has a transaction waiting; the partner is the oldest one waiting there, and a transaction that finds no partner waits until the run ends.",
            resources: "It takes nothing and gives nothing back: a rendezvous is not a handover, and both transactions keep the facility they seized and the storage units they entered. The transaction carries the label of the block it was matched at, and that label is what a `GateM` reads.",
            gotchas: "It needs two blocks: the conjugate is the `RotuloPropio` of the other `Match`, and a `Match` whose `RotuloMatch` is its own label never pairs (its transactions wait there forever). The model does not compile when the conjugate label does not exist (`MATCH <name> refers to unknown label \"BBB\"`). The original pairs only the transactions of the same assembly set; the engine pairs any two that wait, one at each side, the oldest first.",
        },
    ),
    (
        "RemoveMe",
        Semantics {
            flow: "The transaction takes itself out of the group `RemoveMe.Grupo` and goes on to `Salida1`.",
            blocking: "It blocks nothing: the transaction is out of the group in the same instant it arrives.",
            resources: "It takes nothing and gives nothing back; what it gives back is its own place in the group. The group is created by the removal even when nobody was in it, and the transaction keeps the facility it seized and the storage units it entered.",
            gotchas: "The `Salida2` output ('No en [Grupo]') is never used by the engine: the transaction goes on by `Salida1` even when it was not in the group. The statement the engine writes is `REMOVE <grupo>`, without the mode letter of the comparison that the `RemoveTrans` class writes.",
        },
    ),
    (
        "RemoveNumber",
        Semantics {
            flow: "The transaction takes the value of `RemoveNumber.Numero` out of the numeric group `RemoveNumber.Grupo` and goes on to `Salida1`; when the group does not hold that number it goes on to `Salida2` ('No existe el Nº'), or to `Salida1` when that output is not connected.",
            blocking: "It blocks nothing: the number is looked up and taken out at once, and the transaction goes on either way.",
            resources: "It takes nothing and gives nothing back; what it gives back is the number it removes from the group. Only the first occurrence of the number leaves the group.",
            gotchas: "It is the numeric group: the class never touches a group of transactions. `RemoveNumber.Op` is not read, the number is always compared by equality.",
        },
    ),
    (
        "RemoveTrans",
        Semantics {
            flow: "The transaction takes out of the group `RemoveTrans.Grupo` the members whose `RemoveTrans.CompParam` equals `RemoveTrans.CompValue`, at most `RemoveTrans.Cantidad` of them (all of them when `RemoveTrans.Modo` is `Todos`), and goes on to `Salida1`; when it removed nobody and `Salida2` ('No rmv') is connected, it goes on by that one.",
            blocking: "It blocks nothing: the members leave the group at once and the transaction goes on. The members that leave the group keep waiting where they are.",
            resources: "It takes nothing and gives nothing back; what it gives back is the membership of the members it removes. The removed members are not resumed: they keep waiting (normally inside a `Link`) until an `Unlink` releases them.",
            gotchas: "The removed members are detached and not resumed: resuming them here would run them through the remover's own output (the counts of `TP10` agree only under the reading that leaves them waiting). The comparison is always by parameter and by equality (`RemoveTrans.CompOp` and `RemoveTrans.CompOpcion` are not read), and the members are taken in the order the group holds them.",
        },
    ),
    (
        "Return",
        Semantics {
            flow: "The transaction gives the facility `Return.Facility` back and goes on to `Salida1`; the engine compiles the class into a `RELEASE` statement.",
            blocking: "It blocks nothing: the facility is freed in the same instant and the transaction goes on.",
            resources: "It gives back the facility and nothing else: the facility becomes free, its owner is cleared, and the first waiter (by priority and then by arrival) takes it. A transaction that ends does not give the facility back: the `Return` is what gives it back.",
            gotchas: "The class is a `RELEASE`, not the `RETURN` of GPSS: it does not give the facility back to the transaction that had it before a preemption. The `RETURN` that does that is the one the `FacilityInt` class writes by itself.",
        },
    ),
    (
        "Unlink",
        Semantics {
            flow: "The transaction takes out of the chain `Unlink.Cadena` the members whose `Unlink.CompParam` equals `Unlink.CompValue`, at most `Unlink.Cantidad` of them (all of them when `Unlink.Modo` is `Todos`), and the members that leave go out by `SalidaAlt`; the transaction that unlinked goes on to `Salida1`, or to `Salida2` ('No desencadenó') when it could not unlink anybody and that output is connected.",
            blocking: "It blocks nothing: it takes the members it finds and goes on, and the members it takes go on by their own output.",
            resources: "It gives the chain back the members it takes: the size of the chain goes down and the integral of the size of the chain over the time (the `AVE.TIME` of the report) counts the wait they did there.",
            gotchas: "The model does not run unless the `SalidaAlt` output carries a label: the original refuses a model whose `UNLINK` has no operand B ('El parámetro B no debe ser nulo'). The comparison is always by parameter and by equality (`Unlink.CompOp` and `Unlink.CompOpcion` are not read), the parameter that the original returns to the unlinking transaction is not implemented, and the members that leave go out by `Salida1` when `SalidaAlt` is not connected (and leave the run when that output is not connected either). The link indicator of the chain stays on: the engine never clears it, so a transaction that links later waits in the chain even when it is empty.",
        },
    ),
];
