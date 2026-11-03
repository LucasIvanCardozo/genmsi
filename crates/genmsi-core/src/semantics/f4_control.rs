//! The contracts of the family: control.
//!
//! The classes of this family do not carry the transaction from one block to
//! the next: they count, they compare, they look for a member of a group,
//! they interrupt a facility and they decide where the transaction goes next.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Alter",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the block altered at least one member of the group, and to `Salida2` (\"No alteró\") when it altered none. The comparison comes first and the members that match are the ones altered: `Alter.CompParam` is compared with `Alter.CompValue` with the operator `Alter.CompOp`, and the members that match get the value `Alter.AltValor` in the parameter `Alter.AltParam`.",
            blocking: "It blocks nothing: the members are altered while the transaction goes on, and a member keeps its own flow (a member that waits at a `Match` or at a gate keeps waiting there).",
            resources: "It takes nothing and gives nothing back; it writes a parameter in the members it alters, at most `Alter.Cantidad` of them (one at least), in the order the group holds them.",
            gotchas: "`Alter.Modo` is not read by the engine: with `Todos` it still alters at most `Alter.Cantidad` members, while the original ignores the limit when the mode is `Todos`. The engine always alters a parameter and always compares a parameter (`Alter.AltOpcion` and `Alter.CompOpcion` are ignored), so the priority mode of the original is not implemented. `Alter.CompOp` is a relational operator and it is required: a block without it does not compile.",
        },
    ),
    (
        "Counter",
        Semantics {
            flow: "The transaction goes on to `Salida1`: the block counts and lets it continue. The count increments the savevalue `Counter.SaveValue` by `Counter.Incremento` (one by default) and copies the new value into the parameter `Counter.Param`.",
            blocking: "It blocks nothing: the count happens at once and the transaction goes on.",
            resources: "It takes nothing and gives nothing back; it writes the savevalue `Counter.SaveValue`, which the whole run shares, and the parameter `Counter.Param` of the transaction that arrives.",
            gotchas: "The class does not count the members of a group: the count is a savevalue the block increments, and the one-line description the catalogue publishes (\"counts the arrivals of the group `Param`\") does not say what the engine does. `Counter.SaveValue` is required and `Counter.Param` is optional: without it the count stays in the savevalue. The report writes two rows for a `Counter`, the `SAVEVALUE` that counts and the `ASSIGN` that copies the counter into the parameter.",
        },
    ),
    (
        "ExamineNumber",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the number `ExamineNumber.Numero` is a member of the numeric group `ExamineNumber.Grupo`, and to `Salida2` when it is not. It is a test and not a wait: the transaction goes on at once, either way.",
            blocking: "It blocks nothing: the membership is checked at once and the transaction continues; it does not wait for the group to hold the number.",
            resources: "It takes nothing and gives nothing back; it only reads the numeric group `ExamineNumber.Grupo`.",
            gotchas: "The group is the numeric one a `JoinNumber` fills with its `Numero`; a `JoinMe` fills a group of transactions, and the `Examine` class is the one that examines the transaction itself. The one-line description the catalogue publishes says it waits until the group holds the number; the engine branches at once.",
        },
    ),
    (
        "GateM",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the label it carries is the one `GateM.Rotulo` names (with `op` = `Maching`), and to `Salida2` (\"fallo\") when it is not (with `op` = `NotMaching` the two are swapped). The label is the one a `Match` leaves on the transaction: the label of the block it was matched at.",
            blocking: "It blocks only when its failure output `Salida2` is not connected: then the transaction waits in the block, and every change of a logic switch, a storage or a facility retries the waiters. A `GateM` that waits never opens, because the label of a transaction does not change: the transaction stays there until the run ends.",
            resources: "It takes nothing and gives nothing back; it only reads the label the transaction carries. The attempt that fails is counted on a logic switch that carries the name of `GateM.Rotulo`.",
            gotchas: "A `GateM` whose `Rotulo` does not match the label of the transaction holds it forever when `Salida2` is not connected (the benches of the original leave it unconnected): connect `Salida2` when the transaction has to go on. The engine writes the statement of the gate as `GATE M <rótulo>`, the letter of the label, and not the `LS` of a logic switch.",
        },
    ),
    (
        "Loop",
        Semantics {
            flow: "The transaction goes on to the port that leaves the loop, `Salida1` (ioid 2), when the count is over, and to the port that repeats, `Salida2` (ioid 3, the caption `loop`), when it is not: the block decrements `Loop.Parametro` by one on every pass and repeats while what is left is greater than zero. A parameter of n gives n passes through the `Loop` block and n-1 passes through the body the `loop` port feeds.",
            blocking: "It blocks nothing: the loop is a jump and not a wait; the transaction goes on at once through one port or the other.",
            resources: "It takes nothing and gives nothing back; it writes the parameter `Loop.Parametro` of the transaction, which it leaves at zero (below zero when it started below).",
            gotchas: "The two ports are easy to invert, and the model loses its repetitions without any error: the port that repeats is the second one (`Salida2`, ioid 3, the caption `loop`) and the port that leaves the loop is the first one (`Salida1`, ioid 2). When the `loop` port is not connected the repeat falls back to the first port, so the loop runs once.",
        },
    ),
    (
        "Mark",
        Semantics {
            flow: "The transaction goes on to `Salida1`: the block writes the hour of the run in the parameter `Mark.Parametro` and lets the transaction continue.",
            blocking: "It blocks nothing: the writing happens at once.",
            resources: "It takes nothing and gives nothing back; it writes the parameter `Mark.Parametro` of the transaction with the current time of the run.",
            gotchas: "The hour it writes is the model clock, the time the transaction is there, and not the age of the transaction: the class compiles into a plain assignment. `Mark.Parametro` is required: a block without it does not compile.",
        },
    ),
    (
        "Preempt",
        Semantics {
            flow: "The transaction takes the facility `Preempt.Facility` and goes on to `Salida1`. The owner of the facility is interrupted: the time it had left is saved in its parameter `Preempt.Remanente` and it goes on by the interrupt output `SalidaAlt` (\"Propietaria\") when that port is connected.",
            blocking: "It blocks when it may not take the facility: with `ModoInterrupcion` = `Prioridad` (the default) the owner keeps the facility when its priority is not lower than the preemptor's, and the transaction waits in the chain of delay of the facility, as a `Seize` that finds it busy does. With `Incondicional` the preemption happens whatever the priorities are.",
            resources: "It takes the facility and keeps it: the preemptor becomes its owner and the facility is busy until a `Return` (a `Release`) gives it back. An owner that was not routed to `SalidaAlt` does not get the facility back on its own: the `Return` resumes it with the time it had left.",
            gotchas: "With `ModoArrebato` = `Remover` the interrupted owner is removed from the simulation instead of going on; with `NoRemover` (the default) it waits for the `Return`, or goes on by `SalidaAlt` when that port is connected. Two transactions with the same priority never preempt each other in the default mode: the preemptor waits, and the block counts the attempt in its RETRY.",
        },
    ),
    (
        "Scan",
        Semantics {
            flow: "The transaction goes on to `Salida1` (\"Escaneó\") when the block finds a member of the group, and to `Salida2` (\"No escaneó\") when it finds none. The member it finds is the one with the smallest value of `Scan.CompParam` when `Scan.CompOp` is `Minimo`, and the first member whose `Scan.CompParam` equals `Scan.CompValue` otherwise; the value of the parameter `Scan.DeParam` of that member is copied into the parameter `Scan.EnParam` of the transaction that scans.",
            blocking: "It blocks nothing: the search is made at once and the transaction goes on, whether the member was found or not.",
            resources: "It takes nothing and gives nothing back; it reads the group `Scan.Grupo` and writes the parameter `Scan.EnParam` of the transaction that scans.",
            gotchas: "With `Scan.CompOp` = `Minimo` the engine ignores `Scan.CompValue`, and with any other value (`Maximo` included) it looks for a member whose `Scan.CompParam` equals `Scan.CompValue`: `Maximo` does not pick the largest member. The scan only sees the members that are still alive: a member that ended is skipped.",
        },
    ),
    (
        "Select",
        Semantics {
            flow: "The transaction goes on to `Salida1` and the number of the entity it chose is written in the parameter `Select.Param`. The engine walks the entities of the class `Select.ClaseSNA` from `Select.minimo` to `Select.maximo` and picks the one with the smallest value when `Select.op` is `Minimo` and the one with the largest when it is `Maximo` (the value of a queue is its content, the one of a storage the units in use, the one of a facility its busy flag).",
            blocking: "It blocks nothing: the selection is made at once, and when there is nothing to select the transaction goes on with the parameter unchanged.",
            resources: "It takes nothing and gives nothing back; it reads the entities of the range and writes the parameter `Select.Param` of the transaction that selects.",
            gotchas: "The engine walks the queues, the storages, the facilities and the logic switches; the other classes of the original's list (the chains, the groups, the labels, the parameters, the savevalues, the variables and the functions) are named but not walked, and a selection of one of them finds nothing. The class declares one output only: a selection that finds nothing goes on by `Salida1`.",
        },
    ),
    (
        "SelectF",
        Semantics {
            flow: "The transaction goes on to `Salida1` with the number of the facility it chose in the parameter `SelectF.Param`, and to `Salida2` (\"No selec\") when every facility of the range is busy. The engine picks the first free facility between `SelectF.minimo` and `SelectF.maximo`.",
            blocking: "It blocks nothing: a busy facility is not waited for, the block simply goes on by the failure output.",
            resources: "It takes nothing and gives nothing back; it reads the state of the facilities of the range and writes the parameter `SelectF.Param` of the transaction that selects.",
            gotchas: "`SelectF.op` is not read by the engine: it always picks a free facility, so with `Ocupado`, which asks the original for an occupied one, it picks a free one anyway.",
        },
    ),
    (
        "Test",
        Semantics {
            flow: "The transaction goes on to `Salida1` (\"Verdadero\") when the comparison holds and to `Salida2` (\"Falso\") when it does not. The comparison is between `Test.1_Valor` and `Test.3_Valor` with the operator `Test.2_OP`.",
            blocking: "It blocks nothing: the comparison is made at once and the transaction goes on by one of the two ports.",
            resources: "It takes nothing and gives nothing back; it only reads the two values.",
            gotchas: "The operator is required: a `Test` without `Test.2_OP` does not compile (\"unknown comparison operator\"), and the engine takes only the relational operators (`=`, `!=`, `<`, `<=`, `>`, `>=`). The two values are expressions, so a parameter of the transaction can be compared (`P:Nombre`).",
        },
    ),
];
