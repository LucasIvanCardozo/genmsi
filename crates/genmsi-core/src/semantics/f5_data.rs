//! The contracts of the family: the selection and the data.
//!
//! The classes of this family are the ones that carry a value and the ones
//! that pick an entity: the parameter a transaction carries, the save value
//! and the logic switch the model keeps, the note the reader leaves, the
//! table that counts and the three selections of the original.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Assign",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the block writes the parameter `Assign.Parametro` of the transaction that arrives and changes nothing else.",
            blocking: "It blocks nothing: the value is computed and written in the same instant. `Assign.Valor` is an expression evaluated for every arrival (a parameter with `P:`, a save value with `X:`, a function with `FN:`).",
            resources: "It takes nothing and gives nothing back: the parameter belongs to the transaction, and a parameter the transaction does not have yet holds zero. The transaction keeps the facility it seized and the storage units it entered.",
            gotchas: "`Assign.Funcion` is ignored: the engine never multiplies the value by a function and the compiled statement is `ASSIGN P,3`. The operator `Assign.Operador` is `=`, `+` or `-`; any other one (`*`, `/`) does not compile (`unknown assignment operator`), and `Assign.Valor` is required.",
        },
    ),
    (
        "InitiaSV",
        Semantics {
            flow: "It is a definition and not a block of the flow: the class has no ports, it compiles to no statement and it has no row in the report. It gives the save value `InitiaSV.SaveValue` the initial value `InitiaSV.Valor`.",
            blocking: "It blocks nothing: the value is installed before the first transaction of the run exists.",
            resources: "It installs the save value at the start of the run, so the first use of it already finds that value, and it takes nothing from the transaction. The value stays until a `SaveValue` block changes it.",
            gotchas: "`InitiaSV` only sets the initial value: a save value is created where it is used, and one that no `InitiaSV` initializes starts at zero. `InitiaSV.Valor` is a whole number and an empty one means zero; a save value that the model initializes and never uses still appears in the report.",
        },
    ),
    (
        "InitialLS",
        Semantics {
            flow: "It is a definition and not a block of the flow: the class has no ports, it compiles to no statement and it has no row in the report. It gives the logic switch `InitialLS.LogicSwich` its initial state.",
            blocking: "It blocks nothing: the state is installed before the first transaction of the run exists.",
            resources: "It installs the switch at the start of the run: `InitialLS.Valor` = `Set` leaves it set and any other value leaves it reset. It takes nothing from the transaction.",
            gotchas: "The state is read from the property `Valor` (`Set` sets the switch, `Reset` and any other value reset it); a file that carries it in `Op` is read too. A switch that no `InitialLS` initializes starts reset: it is created at its first use.",
        },
    ),
    (
        "Logic",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the block only changes the state of the switch `Logic.LogicSwich`.",
            blocking: "It blocks nothing: the state changes in the same instant. The change releases the transactions that wait at a gate that asks for that switch, because every change of a switch retries the waiters of the gates.",
            resources: "It takes nothing and gives nothing back: the switch belongs to the model and the transaction keeps the facility it seized and the storage units it entered.",
            gotchas: "The operator `Logic.Op` is `Set` or `Reset`, and the engine also reads the original's third operator, `Invertir` (the listing writes `LOGIC I S1`): any value that is neither `Set` nor `Invertir` resets the switch. The name of the switch may be indirect (`*NroSemaforos`), and a switch that no `InitialLS` initializes is created reset at its first mention.",
        },
    ),
    (
        "Note",
        Semantics {
            flow: "It is a comment on the canvas: the class has no ports and the engine compiles nothing for it, so no transaction goes through it and it has no row in the report.",
            blocking: "It blocks nothing: it takes no part in the run.",
            resources: "It takes nothing and gives nothing back.",
            gotchas: "A `Note` cannot be linked (it has no ports) and it appears neither in the listing of the program nor in the report: it is a comment for the reader of the model, not a block of it.",
        },
    ),
    (
        "QTable",
        Semantics {
            flow: "It is a definition and not a block of the flow: the class has no ports, it compiles to no statement and it has no row in the report. It declares the queue table `QTable.NombreTabla` for the queue `QTable.Queue`.",
            blocking: "It blocks nothing: the table is written by the queue itself, when a transaction leaves it.",
            resources: "It takes nothing and gives nothing back. The table records the time each transaction waited in the queue, at the `Depart` that takes it out; the classes are `QTable.Cantidad` in number, `QTable.TamFrecuencia` wide each, from `QTable.PrimerIntervalo` on.",
            gotchas: "The table is written at the `Depart` of the queue and not when the transaction joins it: a transaction that ends without a `Depart` is never tabulated. A `Tabulate` block cannot write into it: it only writes into a table declared with `Table`.",
        },
    ),
    (
        "SaveValue",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the block writes the save value `SaveValue.SaveValue` and nothing else.",
            blocking: "It blocks nothing: the value is computed and written in the same instant.",
            resources: "The save value belongs to the model and not to the transaction: it is created where it is used (at its first use it holds zero) and it keeps its value until another block changes it. It takes nothing from the transaction, which keeps its parameters, its facility and its storage units. The value of a save value is read with the prefix `X:` (the original writes it `X$Count`).",
            gotchas: "`InitiaSV` only sets the initial value: a save value that no `InitiaSV` initializes starts at zero. The operator `SaveValue.Op` is `=`, `+` or `-` and any other one does not compile; `SaveValue.Valor` is required. A selection whose `ClaseSNA` is `SV` does not read the value of a save value: the engine walks the storages there (see `SelectSNA`).",
        },
    ),
    (
        "SelectL",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the search finds a switch, and it goes on by that same first output when it finds none: the second output ('No selec') is never taken.",
            blocking: "It blocks nothing: the search is done in the same instant.",
            resources: "It takes nothing and gives nothing back: it writes the number of the switch it picked into the parameter `SelectL.Param`. The switches are the numbered ones (`1`, `2`, ...), and the search covers the numbers from `SelectL.minimo` to `SelectL.maximo` (both are 1 when the model does not set them, and the range is taken in order).",
            gotchas: "The operator `SelectL.op` is `Set` (a switch that is set) or `Reset` (one that is not); any other value means `Set`. When the search finds nothing the parameter is not written at all (it keeps the value it had), and every switch the search mentions is created reset. The listing writes the second output as the destination of the failure (`SELECT LS param,1,3,,,A_T2`), but the engine never takes it.",
        },
    ),
    (
        "SelectS",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the search finds a storage, and it goes on by that same first output when it finds none: the second output ('No selec') is never taken.",
            blocking: "It blocks nothing: the search is done in the same instant.",
            resources: "It takes nothing and gives nothing back: it writes the number of the storage it picked into the parameter `SelectS.Param`. The storages are the numbered ones (`1`, `2`, ...), and the search covers the numbers from `SelectS.minimo` to `SelectS.maximo` (both are 1 when the model does not set them, and the range is taken in order).",
            gotchas: "The operator `SelectS.op` is the state the storage must be in: `Vacio` (empty), `NoVacio` (not empty), `Lleno` (full), `NoLleno` (not full), `Disponible` (available) and `NoDisponible` (not available); any other value means `NoLleno`. The engine has no statement that takes a storage out of the available state, so `Disponible` matches every storage and `NoDisponible` matches none. A storage the model does not declare holds nothing: it is empty and not full, so it matches `Vacio` and `NoLleno`. The listing writes the second output as the destination of the failure, but the engine never takes it.",
        },
    ),
    (
        "SelectSNA",
        Semantics {
            flow: "The transaction goes on to `Salida1` when the search finds an entity, and it goes on by that same first output when it finds none: the second output ('No selec') is never taken.",
            blocking: "It blocks nothing: the search is done in the same instant.",
            resources: "It takes nothing and gives nothing back: it writes the number of the entity it picked into the parameter `SelectSNA.Param`. The entities are the numbered ones of the family `SelectSNA.ClaseSNA`, and the search covers the numbers from `SelectSNA.minimo` to `SelectSNA.maximo` (both are 1 when the model does not set them). The value compared is the content of the queue (`Q`), the units in use of the storage (`S` and `R`) or the busy flag of the facility (`F`).",
            gotchas: "`SelectSNA.op` is ignored: the comparison is always an equality with `SelectSNA.valorComp`, and without that property the value compared is zero. The class `SelectSNA.ClaseSNA` picks the family: `Q` the queues, `S` and `R` the storages, `F` the facilities, `LS`/`LR` the logic switches; only the queue, the storage and the facility carry a value to compare, so for every other class the value compared is zero. `R` (the units left) is compared as the units in use: the engine reads the same number for `S` and for `R`. `SV` is a storage (the units in use) and not a save value: a selection with `ClaseSNA=SV` never reads the value of a save value, and the prefix that reads one is `X:`.",
        },
    ),
    (
        "Tabulate",
        Semantics {
            flow: "The transaction goes on to `Salida1` at once: the block only records a value in the table.",
            blocking: "It blocks nothing: the sample is taken in the same instant.",
            resources: "It takes nothing and gives nothing back: it records one sample in the table `Tabulate.NombreTabla`, and the value of the sample is the expression of the table's `Atributo` evaluated at this block, not when the table was declared.",
            gotchas: "The table has to exist: a `Tabulate` whose `NombreTabla` no `Table` declares records nothing, and the run says `TABULATE X has no TABLE definition`. The weight `Tabulate.Peso` is written in the listing (`TABULATE TBL1,5`) but the engine counts one sample per arrival: the weight does not multiply the frequency.",
        },
    ),
];
