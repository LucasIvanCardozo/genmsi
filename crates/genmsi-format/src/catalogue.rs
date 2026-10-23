//! The block catalogue: the classes a block can be created from.
//!
//! The shape of every class comes from the original editor's own class
//! library (`Genmsi_20230711.backup/gClasses.xml`): the ports it declares
//! (with the `ioid` and `name` the file uses) and the properties it declares
//! (with the hint and the default value the macro code carries). The
//! property *types* come from the original's macro configuration
//! (`gmacroconfig.gmcfg`), and the classes the reference models in
//! `fixtures/models` use are pinned by
//! `tests/catalogue_matches_the_reference_models.rs`.
//!
//! `title` is what the interface shows (neutral Spanish); the `class` name is
//! the model's own identifier and is never translated.

use crate::error::FormatError;
use crate::model::{Block, Layout, Port, Property, SectionKind};

/// One connection point of a catalogue class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortSpec {
    /// Port id inside the block (`ioid`).
    pub ioid: u32,
    /// Port name as the reference models declare it (`Entrada1`, `Salida1`).
    pub name: &'static str,
    /// Port caption, percent-encoded as the models store it.
    pub caption: &'static str,
}

/// One declared property of a catalogue class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertySpec {
    /// Property name as declared by the block schema (`Media`, `Queue`, ...).
    pub name: &'static str,
    /// Tooltip exactly as the file stores it (percent-encoded).
    pub hint: &'static str,
    /// Declared type (`TSNA`, `EntityUse`, `Listed`, `PosInteger`, ...).
    pub ty: &'static str,
    /// Declared subtype (`SNA`, `Queue`, `op_AddSub`, ...).
    pub subtype: &'static str,
    /// Whether the property defines a new entity.
    pub unique: bool,
}

/// The shape of one block class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassSpec {
    /// Class name as the model file uses it (`moduleclass`).
    pub class: &'static str,
    /// What the interface shows for the class (neutral Spanish).
    pub title: &'static str,
    /// Input ports, in the order the models declare them.
    pub inputs: &'static [PortSpec],
    /// Output ports, in the order the models declare them.
    pub outputs: &'static [PortSpec],
    /// Declared properties, in schema order.
    pub properties: &'static [PropertySpec],
}

/// Every class the editor offers, in catalogue order.
pub static CLASSES: &[ClassSpec] = &[
    ADVANCE, ALTER, ASSEMBLE, ASSIGN, BUFFER, CLOCK, COUNTER, DEPART, ENTER, EXAMINE, EXAMINENUMBER,
    FACILITYINT, FACILITYUSE, FUNCTION, FUNCTIONDIST, FUNCTIONEXPON, GATEF, GATEL, GATEM, GATES,
    GATHER, GENERATE, INITIASV, INITIALLS, JOINME, JOINNUMBER, LEAVE, LINK, LOGIC, LOOP, MARK,
    MATCH, MULTIFURCACION, MULTIFURCACIONP, NOTE, PREEMPT, PRIORITY, QTABLE, QUEUE, RELEASE,
    REMOVEME, REMOVETRANS, RETURN, SAVEVALUE, SCAN, SEIZE, SELECT, SELECTF, SELECTL, SELECTS,
    SELECTSNA, SPLIT, STORAGEDEF, STORAGEUSE, TABLE, TABULATE, TERMINATE, TEST, TRANSFER, UNLINK,
    VARIABLE, VARIABLEA, VARIABLEN,
];

/// The value a new block of a class comes with, per property.
///
/// The row is `(class, property, value)`. The value is the raw one: the
/// text the file carries, percent-encoded the way the original writes it
/// (a `=` is `%3D`, an accented letter is its CP1252 byte, a `%` is
/// `%25`). A property the original leaves empty has no row: an empty
/// default is a default too, and writing it changes nothing.
///
/// Every row comes from the original's own class library
/// (`Genmsi_20230711.backup/gClasses.xml`): the macro code of a class
/// (its `MAKE` line) declares its properties with the default value the
/// class carries. The rows that come from somewhere else say so:
///
/// - the `FunctionExpon` class names its random seed by number (`RN:2`),
///   and the property that carries it holds that number;
/// - the two multifurcations name their properties in the macro code
///   without declaring them, so their values come from the model the
///   original ships, the one that exercises the class;
/// - the `SelectSNA` class declares no operator, and the original falls
///   back to the first option of the list.
///
/// The `PriorityOption` properties are the one place where the table does
/// not repeat the original's spelling: the original writes `Parámetro`
/// and the list of options of the catalogue is ASCII-only, so the default
/// is the same option written `Parametro`.
pub static DEFAULTS: &[(&str, &str, &str)] = &[
    // The `Advance` class waits a mean of ten units, with a spread of five.
    ("Advance", "Media", "10"),
    ("Advance", "Modificador", "5"),
    // The `Alter` class alters the transactions of a group.
    ("Alter", "AltOpcion", "Parametro"),
    ("Alter", "AltParam", "param"),
    ("Alter", "AltValor", "0"),
    ("Alter", "Cantidad", "1"),
    ("Alter", "CompOp", "%3D"),
    ("Alter", "CompOpcion", "Parametro"),
    ("Alter", "CompParam", "otro"),
    ("Alter", "CompValue", "1"),
    ("Alter", "Grupo", "ungrupo"),
    ("Alter", "Modo", "Algunos"),
    // The `Assemble` class joins one transaction of the group.
    ("Assemble", "Cantidad", "1"),
    // The `Assign` class assigns a value to a parameter.
    ("Assign", "Operador", "%3D"),
    ("Assign", "Parametro", "Param"),
    ("Assign", "Valor", "0"),
    // The clock counts in units of time: the original gives it a whole hour.
    ("Clock", "Frecuencia", "3600"),
    // The `Counter` class counts and keeps the count in a save value.
    ("Counter", "Incremento", "1"),
    ("Counter", "Param", "Nro"),
    ("Counter", "SaveValue", "Count"),
    // The `Depart` class leaves a queue.
    ("Depart", "Queue", "Qrecurso"),
    // The `Enter` class enters a storage.
    ("Enter", "Cantidad", "1"),
    ("Enter", "Storage", "AStorage"),
    // The `Examine` class asks whether the transaction belongs to a group.
    ("Examine", "Grupo", "UnGrupo"),
    // The `ExamineNumber` class asks whether a number belongs to a group.
    ("ExamineNumber", "Grupo", "UnGrupo"),
    ("ExamineNumber", "Numero", "0"),
    // The `FacilityInt` class interrupts a facility.
    ("FacilityInt", "Facility", "Recurso"),
    ("FacilityInt", "Media", "10"),
    ("FacilityInt", "Modificador", "5"),
    ("FacilityInt", "ModoArrebato", "NoRemover"),
    ("FacilityInt", "ModoInterrupcion", "Prioridad"),
    ("FacilityInt", "Queue", "ColaInt"),
    ("FacilityInt", "Remanente", "Remanente"),
    // The `FacilityUse` class uses a facility.
    ("FacilityUse", "Facility", "Recurso"),
    ("FacilityUse", "Media", "10"),
    ("FacilityUse", "Modificador", "5"),
    ("FacilityUse", "Queue", "Cola"),
    // The `Function` class defines a function of an attribute.
    ("Function", "Atributo", "P%3AParam"),
    ("Function", "NombreFunc", "Func"),
    ("Function", "Tabla", "%7B1%7C1%3B2%7C2%7D"),
    ("Function", "Tipo", "Discreta"),
    // The `FunctionDist` class defines a function of distribution.
    ("FunctionDist", "Distribucion", "%7B50%25%3B50%25%7D"),
    ("FunctionDist", "NombreDist", "FuncDist"),
    ("FunctionDist", "Semilla", "3"),
    // The `FunctionExpon` class defines an exponential function.
    // The random seed the macro code names (`RN:2`): the seed number is
    // the value of the property that carries it.
    ("FunctionExpon", "2", "2"),
    ("FunctionExpon", "NombreFunc", "Expon"),
    ("FunctionExpon", "Seed", "2"),
    // The `GateF` class asks whether a facility is free.
    ("GateF", "Facility", "Recurso"),
    ("GateF", "op", "Libre"),
    // The `GateL` class asks about the state of a logic switch.
    ("GateL", "LogicSwich", "1"),
    ("GateL", "op", "Set"),
    // The `GateM` class asks whether the transaction matches a label.
    ("GateM", "Rotulo", "Rotulo"),
    ("GateM", "op", "Maching"),
    // The `GateS` class asks about the state of a storage.
    ("GateS", "Storage", "Almacen"),
    ("GateS", "op", "NoLleno"),
    // The `Gather` class gathers the transactions of a group.
    ("Gather", "Cantidad", "1"),
    // The generator of the original has a mean of 60 and a spread of 30. The
    // spread is what makes it a random source: without it the interarrival is
    // constant, every copy of the generator produces the very same arrivals and
    // the seed has nothing to say.
    ("Generate", "Media", "60"),
    ("Generate", "Modificador", "30"),
    // The `InitiaSV` class initializes a save value.
    ("InitiaSV", "SaveValue", "1"),
    ("InitiaSV", "Valor", "0"),
    // The `InitialLS` class initializes a logic switch.
    ("InitialLS", "LogicSwich", "1"),
    ("InitialLS", "Valor", "Set"),
    // The `JoinMe` class joins a group.
    ("JoinMe", "Grupo", "UnGrupo"),
    // The `JoinNumber` class adds a number to a group.
    ("JoinNumber", "Grupo", "UnGrupo"),
    ("JoinNumber", "Numero", "0"),
    // The `Leave` class leaves a storage.
    ("Leave", "Cantidad", "1"),
    ("Leave", "Storage", "AStorage"),
    // The `Link` class chains the transaction.
    ("Link", "Cadena", "Cadena"),
    ("Link", "Modo", "FIFO"),
    // The `Logic` class sets a logic switch.
    ("Logic", "LogicSwich", "1"),
    ("Logic", "Op", "Set"),
    // The `Loop` class decrements a parameter on every iteration.
    ("Loop", "Parametro", "Param"),
    // The `Mark` class writes the hour in a parameter.
    ("Mark", "Parametro", "Param"),
    // The `Match` class synchronizes the transactions of a label.
    ("Match", "RotuloMatch", "BBB"),
    ("Match", "RotuloPropio", "AAA"),
    // The `Multifurcacion` class creates the two functions it needs and
    // splits the traffic in five equal shares.
    // The macro code of the two multifurcations names these properties
    // without declaring them: the values come from the model the original
    // ships, the one that exercises the class.
    ("Multifurcacion", "MultifurcacionCH", "MultifurcacionCH"),
    ("Multifurcacion", "MultifurcacionGO", "MultifurcacionGO"),
    ("Multifurcacion", "Prob1", "20%25"),
    ("Multifurcacion", "Prob2", "20%25"),
    ("Multifurcacion", "Prob3", "20%25"),
    ("Multifurcacion", "Prob4", "20%25"),
    ("Multifurcacion", "Prob5", "20%25"),
    // The `MultifurcacionP` class routes by a parameter: it creates one
    // function and splits the traffic in five equal shares.
    ("MultifurcacionP", "MultifurcacionGO", "MultifurcacionGO"),
    ("MultifurcacionP", "Parametro", "Param"),
    ("MultifurcacionP", "Prob1", "20%25"),
    ("MultifurcacionP", "Prob2", "20%25"),
    ("MultifurcacionP", "Prob3", "20%25"),
    ("MultifurcacionP", "Prob4", "20%25"),
    ("MultifurcacionP", "Prob5", "20%25"),
    // The `Preempt` class interrupts a facility: the original's macro code
    // gives the facility, the mode, the parameter and the way the owner goes
    // on these defaults.
    ("Preempt", "Facility", "Recurso"),
    ("Preempt", "ModoArrebato", "NoRemover"),
    ("Preempt", "ModoInterrupcion", "Prioridad"),
    ("Preempt", "Remanente", "Remanente"),
    // The `Priority` class gives the transaction a priority.
    ("Priority", "Prioridad", "0"),
    // The `QTable` class defines the frequency table of a queue.
    ("QTable", "Cantidad", "60"),
    ("QTable", "NombreTabla", "QTabla"),
    ("QTable", "PrimerIntervalo", "0"),
    ("QTable", "Queue", "qRecurso"),
    ("QTable", "TamFrecuencia", "600"),
    // The `Queue` class enters a queue.
    ("Queue", "Queue", "QRecurso"),
    // The `Release` class frees a facility.
    ("Release", "Facility", "Recurso"),
    // The `RemoveMe` class takes the transaction out of its group.
    ("RemoveMe", "Grupo", "UnGrupo"),
    // The `RemoveTrans` class removes the transactions of a group.
    ("RemoveTrans", "Cantidad", "1"),
    ("RemoveTrans", "CompOp", "%3D"),
    ("RemoveTrans", "CompOpcion", "Parametro"),
    ("RemoveTrans", "CompParam", "otro"),
    ("RemoveTrans", "CompValue", "1"),
    ("RemoveTrans", "Grupo", "UnGrupo"),
    ("RemoveTrans", "Modo", "Algunos"),
    // The `Return` class gives the facility back: it is a release.
    ("Return", "Facility", "Recurso"),
    // The `SaveValue` class gives a value to a save value.
    ("SaveValue", "Op", "%3D"),
    ("SaveValue", "SaveValue", "1"),
    ("SaveValue", "Valor", "0"),
    // The `Scan` class scans a group.
    ("Scan", "CompOp", "%3D"),
    ("Scan", "CompOpcion", "Parametro"),
    ("Scan", "CompParam", "otro"),
    ("Scan", "CompValue", "1"),
    ("Scan", "DeOpción", "Parametro"),
    ("Scan", "DeParam", "Param"),
    ("Scan", "EnOpcion", "Parametro"),
    ("Scan", "EnParam", "Param"),
    ("Scan", "Grupo", "UnGrupo"),
    // The `Seize` class takes a facility.
    ("Seize", "Facility", "Recurso"),
    // The `Select` class picks an entity of a class.
    ("Select", "ClaseSNA", "Q"),
    ("Select", "Param", "param"),
    ("Select", "maximo", "10"),
    ("Select", "minimo", "1"),
    ("Select", "op", "Minimo"),
    // The `SelectF` class picks a free facility.
    ("SelectF", "Param", "param"),
    ("SelectF", "maximo", "10"),
    ("SelectF", "minimo", "1"),
    ("SelectF", "op", "Libre"),
    // The three selections of the original: a logic switch, a storage and an
    // entity of a class.
    ("SelectL", "Param", "param"),
    ("SelectL", "maximo", "10"),
    ("SelectL", "minimo", "1"),
    ("SelectL", "op", "Set"),
    ("SelectS", "Param", "param"),
    ("SelectS", "maximo", "10"),
    ("SelectS", "minimo", "1"),
    ("SelectS", "op", "NoLleno"),
    ("SelectSNA", "ClaseSNA", "Q"),
    ("SelectSNA", "Param", "param"),
    ("SelectSNA", "maximo", "10"),
    ("SelectSNA", "minimo", "1"),
    // The `SelectSNA` class declares no operator: the original falls back
    // to the first option of the list.
    ("SelectSNA", "op", "%3D"),
    ("SelectSNA", "valorComp", "%2A1"),
    // The `Split` class clones the transaction.
    ("Split", "Cantidad", "1"),
    // The `StorageDef` class declares a storage.
    ("StorageDef", "Storage", "Almacen"),
    ("StorageDef", "Tamaño", "10"),
    // The `StorageUse` class uses a storage for a while.
    ("StorageUse", "Media", "10"),
    ("StorageUse", "Modificador", "5"),
    ("StorageUse", "Queue", "qAlmacen"),
    ("StorageUse", "Storage", "Almacen"),
    // The `Table` class defines the frequency table of an attribute.
    ("Table", "Atributo", "P%3AAtrib"),
    ("Table", "Cantidad", "10"),
    ("Table", "NombreTabla", "Tabla"),
    ("Table", "PrimerIntervalo", "0"),
    ("Table", "TamFrecuencia", "10"),
    // The `Tabulate` class tabulates a table.
    ("Tabulate", "NombreTabla", "Tabla"),
    // The `Test` class compares two values.
    ("Test", "1_Valor", "1"),
    ("Test", "2_OP", "%3D"),
    ("Test", "3_Valor", "1"),
    // The `Unlink` class unchains the transactions of a chain.
    ("Unlink", "Cadena", "Cadena"),
    ("Unlink", "Cantidad", "1"),
    ("Unlink", "CompOp", "%3D"),
    ("Unlink", "CompOpcion", "Parametro"),
    ("Unlink", "CompParam", "Otro"),
    ("Unlink", "CompValue", "0"),
    ("Unlink", "Modo", "Algunos"),
    // The `Variable` class defines a variable.
    ("Variable", "NombreVar", "vari"),
    // The `VariableA` class defines a uniform random variable.
    ("VariableA", "Maximo", "10"),
    ("VariableA", "Minimo", "1"),
    ("VariableA", "NombreVar", "Azar"),
    ("VariableA", "Seed", "80"),
    // The `VariableN` class: a normal variable with its own function.
    ("VariableN", "Desvio", "1"),
    ("VariableN", "Media", "60"),
    ("VariableN", "NombreFunc", "NORMAL"),
    ("VariableN", "NombreVar", "Normal"),
    ("VariableN", "Seed", "90"),
];

/// The default value a new block of `class` comes with for `property`, when
/// the class has one.
pub fn default_value(class: &str, property: &str) -> Option<&'static str> {
    DEFAULTS
        .iter()
        .find(|(c, p, _)| *c == class && *p == property)
        .map(|(_, _, value)| *value)
}

/// Options of a `Listed` property, by its subtype.
///
/// Only ASCII values: the file's encoding is CP1252 and the editor must not
/// write a value it cannot encode the way the original did.
static LISTED_OPTIONS: &[(&str, &[&str])] = &[
    ("op_AddSub", &["+", "-", "="]),
    ("op_Relac", &["=", "<", "<=", ">", ">=", "!="]),
    ("op_RelacMaxMin", &["=", "Minimo", "Maximo"]),
    ("op_MaxMin", &["Minimo", "Maximo"]),
    ("op_Swich", &["Set", "Reset"]),
    ("op_Logic_Swich", &["Set", "Reset"]),
    ("op_Logic_Stor", &["Vacio", "NoVacio", "Lleno", "NoLleno", "Disponible", "NoDisponible"]),
    ("op_Logic_Faci", &["Libre", "Ocupado"]),
    // The gate that waits for a label of a transaction.
    ("op_Logic_Tran", &["Maching", "NotMaching"]),
    ("FuncTypes", &["Continua", "Discreta", "Eval"]),
    ("LinkMode", &["FIFO"]),
    ("OperationMode", &["Algunos", "Todos"]),
    ("PriorityOption", &["Parametro"]),
    // The original's macro configuration (`gmacroconfig.gmcfg`) gives every
    // listed subtype its options: the priority mode is `PR` (the preempting
    // transaction's priority decides) or unconditional, and the way the
    // preempted owner goes on is to remove it or to keep it.
    ("PriorityMode", &["Prioridad", "Incondicional"]),
    ("RemoveMode", &["Remover", "NoRemover"]),
    // The original's `entitySNAclass` list (`gmacroconfig.gmcfg`) has 38
    // classes: each one names an attribute of an entity (the content of a
    // queue, the units in use of a storage, the average time of a chain...)
    // and the `SELECT` statement carries the class as its last operand. The
    // original accepts every one of them.
    (
        "entitySNAclass",
        &[
            "SR", "FR", "CH", "CC", "CA", "QZ", "GN", "W", "N", "SC", "FC", "GT", "QC", "CM", "QM",
            "SM", "QA", "SA", "MP", "FV", "LR", "LS", "RN", "S", "F", "Q", "R", "BV", "FN", "V",
            "SV", "FT", "QT", "QX", "CT", "ST", "X", "P",
        ],
    ),
];

/// Options of a `Listed` property by its subtype; empty for an unknown
/// subtype.
pub fn listed_options(subtype: &str) -> &'static [&'static str] {
    LISTED_OPTIONS
        .iter()
        .find(|(key, _)| *key == subtype)
        .map(|(_, options)| *options)
        .unwrap_or(&[])
}

/// The catalogue entry of a class, or `None` when the class is not offered
/// (custom macros, plain containers, the model root).
pub fn class_spec(class: &str) -> Option<&'static ClassSpec> {
    CLASSES.iter().find(|spec| spec.class == class)
}

/// Build the `Block` of a catalogue class with the given owner and layout.
///
/// The block gets id `0`: the caller assigns the real id. The ports come from
/// the catalogue with empty labels and no port layout, the properties with
/// the default value of their class (see [`DEFAULTS`]) or with an empty one
/// when the class has no default for them. The sections are the ones the
/// block actually has, in the order `Inputs`, `Outputs`, `Props`; a class with
/// no ports and no properties (a note) gets none at all, so the writer emits
/// the self-closing form the original uses.
pub fn build_block(class: &str, owner: u32, layout: Option<Layout>) -> Result<Block, FormatError> {
    let spec = class_spec(class).ok_or_else(|| {
        FormatError::structural(0, format!("clase de bloque desconocida: {class}"))
    })?;
    let mut sections = Vec::new();
    let inputs: Vec<Port> = spec
        .inputs
        .iter()
        .map(|p| Port {
            ioid: p.ioid,
            name: p.name.to_string(),
            caption: p.caption.to_string(),
            mylabel: String::new(),
            layout: None,
        })
        .collect();
    let outputs: Vec<Port> = spec
        .outputs
        .iter()
        .map(|p| Port {
            ioid: p.ioid,
            name: p.name.to_string(),
            caption: p.caption.to_string(),
            mylabel: String::new(),
            layout: None,
        })
        .collect();
    let props: Vec<Property> = spec
        .properties
        .iter()
        .map(|p| Property {
            name: p.name.to_string(),
            hint: p.hint.to_string(),
            raw_value: default_value(spec.class, p.name)
                .map(str::to_string)
                .unwrap_or_default(),
            ty: p.ty.to_string(),
            subtype: p.subtype.to_string(),
            visible: true,
            unique: p.unique,
        })
        .collect();
    if !inputs.is_empty() {
        sections.push(SectionKind::Inputs);
    }
    if !outputs.is_empty() {
        sections.push(SectionKind::Outputs);
    }
    if !props.is_empty() {
        sections.push(SectionKind::Props);
    }
    Ok(Block {
        id: 0,
        class: spec.class.to_string(),
        kind: 'A',
        owner,
        name: spec.class.to_string(),
        caption: String::new(),
        layout,
        header: None,
        inputs,
        outputs,
        links: Vec::new(),
        props,
        sections,
        source: None,
    })
}

// ---------------------------------------------------------------------------
// Class data
//
// The shape of every class comes from the original editor's class library
// (`Genmsi_20230711.backup/gClasses.xml`): the ports its graph declares and
// the properties its macro code declares, with the default value the macro
// code carries. The types come from the original's macro configuration
// (`gmacroconfig.gmcfg`).
// ---------------------------------------------------------------------------

// One ClassSpec per block class the editor offers.
pub static ADVANCE: ClassSpec = ClassSpec {
    class: "Advance",
    title: "Avanzar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingresa" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Media", hint: "Tiempo%20Medio%20de%20espera", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modificador", hint: "Desvio%20o%20Modificador", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static ALTER: ClassSpec = ClassSpec {
    class: "Alter",
    title: "Alterar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "Alter%F3" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20alter%F3" },
    ],
    properties: &[
        PropertySpec { name: "AltOpcion", hint: "Que%20se%20altera", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "AltParam", hint: "Parametro%20a%20alterar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "AltValor", hint: "Valor%20a%20asignar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20transacciones%20a%20alterar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "CompOp", hint: "operador", ty: "Listed", subtype: "op_RelacMaxMin", unique: false },
        PropertySpec { name: "CompOpcion", hint: "Que%20se%20compara", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "CompParam", hint: "Parametro%20a%20comparar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "CompValue", hint: "Valor%20a%20comparar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Grupo", hint: "Grupo%20de%20transacciones", ty: "EntityUse", subtype: "TransacGroup", unique: false },
        PropertySpec { name: "Modo", hint: "Como%20se%20alteran", ty: "Listed", subtype: "OperationMode", unique: false },
    ],
};

pub static ASSEMBLE: ClassSpec = ClassSpec {
    class: "Assemble",
    title: "Ensamblar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Convina" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20transacciones%2C%20del%20mismo%20grupo%20de%20reuni%F3n%2C%20a%20combinar", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static ASSIGN: ClassSpec = ClassSpec {
    class: "Assign",
    title: "Asignar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Funcion", hint: "Funci%F3n%20por%20la%20que%20se%20multiblica%20el%20valor", ty: "EntityUse", subtype: "Function", unique: false },
        PropertySpec { name: "Operador", hint: "Operador%20para%20modificar%20el%20Par%E1metro%20con%20el%20Valor", ty: "Listed", subtype: "op_AddSub", unique: false },
        PropertySpec { name: "Parametro", hint: "Parametro%20a%20asignar%20o%20modificar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "Valor", hint: "Valor%20a%20asignar", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static BUFFER: ClassSpec = ClassSpec {
    class: "Buffer",
    title: "Retener",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[],
};

pub static CLOCK: ClassSpec = ClassSpec {
    class: "Clock",
    title: "Reloj",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Frecuencia", hint: "frecuencia%2C%20en%20unidad%20de%20tiempo%2C%20con%20la%20que%20se%20descuenta%201%20unidad%20del%20START", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static COUNTER: ClassSpec = ClassSpec {
    class: "Counter",
    title: "Contar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso%20al%20m%F3dulo" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Incremento", hint: "Valor%20incremental%20de%20conteo", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Param", hint: "Parmetro%20de%20conteo", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "SaveValue", hint: "SaveValueUtilizado", ty: "EntityUse", subtype: "SaveValue", unique: false },
    ],
};

pub static DEPART: ClassSpec = ClassSpec {
    class: "Depart",
    title: "Salir de cola",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20egresos", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Queue", hint: "Queue%20de%20la%20cual%20se%20egresa", ty: "EntityUse", subtype: "Queue", unique: false },
    ],
};

pub static ENTER: ClassSpec = ClassSpec {
    class: "Enter",
    title: "Ingresar a depósito",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20ingresos", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Storage", hint: "Storage%20al%20cual%20ingresa%20la%20transacci%F3n", ty: "EntityUse", subtype: "Storage", unique: false },
    ],
};

pub static EXAMINE: ClassSpec = ClassSpec {
    class: "Examine",
    title: "Examinar grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "En%20%5BGrupo%5D" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20en%20%5BGrupo%5D" },
    ],
    properties: &[
        PropertySpec { name: "Grupo", hint: "Grupo%20de%20transacciones%20a%20examinar", ty: "EntityUse", subtype: "TransacGroup", unique: false },
    ],
};

pub static EXAMINENUMBER: ClassSpec = ClassSpec {
    class: "ExamineNumber",
    title: "Examinar número",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Grupo", hint: "Grupo%20a%20examinar", ty: "EntityUse", subtype: "NumericGroup", unique: false },
        PropertySpec { name: "Numero", hint: "N%BA%20a%20Examinar%20si%20pertenece%20al%20grupo%20%5BGrupo%5D", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static FACILITYINT: ClassSpec = ClassSpec {
    class: "FacilityInt",
    title: "Interrumpir recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso%20al%20recurso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "SalidaAlt", caption: "Propietaria" },
        PortSpec { ioid: 3, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20interrumpir%20por%20la%20transacci%F3n%20activa.", ty: "EntityUse", subtype: "Facility", unique: false },
        PropertySpec { name: "Media", hint: "Tiempo%20Medio%20de%20uso%20del%20recurso", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modificador", hint: "Desvio%20o%20Modificador%20de%20la%20media", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "ModoArrebato", hint: "Modo%20de%20arrebato%20temporal%20o%20permanete.", ty: "Listed", subtype: "RemoveMode", unique: false },
        PropertySpec { name: "ModoInterrupcion", hint: "Modo%20de%20interrupci%F3n%20o%20por%20prioridad", ty: "Listed", subtype: "PriorityMode", unique: false },
        PropertySpec { name: "Queue", hint: "cola%20del%20recurso%20interrumpido", ty: "EntityUse", subtype: "Queue", unique: false },
        PropertySpec { name: "Remanente", hint: "Par%E1metro%20de%20la%20transacci%F3n%20propietaria%20donde%20se%20guarda%20el%20remanete", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static FACILITYUSE: ClassSpec = ClassSpec {
    class: "FacilityUse",
    title: "Usar recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso%20al%20recurso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Recurso%20utilizado", ty: "EntityUse", subtype: "Facility", unique: false },
        PropertySpec { name: "Media", hint: "Tiempo%20Medio%20de%20uso%20del%20recurso", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modificador", hint: "Desvio%20o%20Modificador%20de%20la%20media", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Queue", hint: "cola%20del%20recurso%20utilizado", ty: "EntityUse", subtype: "Queue", unique: false },
    ],
};

pub static FUNCTION: ClassSpec = ClassSpec {
    class: "Function",
    title: "Definir función",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Atributo", hint: "Variable%20independiente", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "NombreFunc", hint: "Nombre%20de%20la%20Funci%F3n", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "Tabla", hint: "Tabla%20que%20define%20el%20compertamiento%20de%20la%20funci%F3n", ty: "Table", subtype: "", unique: false },
        PropertySpec { name: "Tipo", hint: "Tipo%20de%20Funci%F3n", ty: "Listed", subtype: "FuncTypes", unique: false },
    ],
};

pub static FUNCTIONDIST: ClassSpec = ClassSpec {
    class: "FunctionDist",
    title: "Definir función de distribución",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Distribucion", hint: "Distribuci%F3n%20de%20probabilidades%20para%20cada%20resultado", ty: "FloatPercentList", subtype: "", unique: false },
        PropertySpec { name: "NombreDist", hint: "Nombre%20de%20la%20funci%F3n%20de%20distribuci%F3n", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "Semilla", hint: "n%FAmero%20de%20semilla%20utilizada%20para%20calcular%20los%20n%FAmeros%20aleatorios", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static FUNCTIONEXPON: ClassSpec = ClassSpec {
    class: "FunctionExpon",
    title: "Definir función exponencial",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "2", hint: "", ty: "EntityUse", subtype: "RandomSeed", unique: false },
        PropertySpec { name: "NombreFunc", hint: "Nombre%20de%20la%20Funci%F3n", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "Seed", hint: "Cemilla%20de%20n%FAmero%20aleatorio", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static GATEF: ClassSpec = ClassSpec {
    class: "GateF",
    title: "Verificar recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "fallo" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20evaluar.", ty: "EntityUse", subtype: "Facility", unique: false },
        PropertySpec { name: "op", hint: "Operador%20logico%20de%20facility.", ty: "Listed", subtype: "op_Logic_Faci", unique: false },
    ],
};

pub static GATEL: ClassSpec = ClassSpec {
    class: "GateL",
    title: "Verificar llave lógica",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "fallo" },
    ],
    properties: &[
        PropertySpec { name: "LogicSwich", hint: "Logic%20Swich%20a%20evaluar.", ty: "EntityUse", subtype: "LogicSwich", unique: false },
        PropertySpec { name: "op", hint: "Operador%20logico%20del%20logic%20swich.", ty: "Listed", subtype: "op_Logic_Swich", unique: false },
    ],
};

pub static GATEM: ClassSpec = ClassSpec {
    class: "GateM",
    title: "Verificar etiqueta",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "fallo" },
    ],
    properties: &[
        PropertySpec { name: "Rotulo", hint: "Rotulo%20a%20evaluar.", ty: "EntityUse", subtype: "Label", unique: false },
        PropertySpec { name: "op", hint: "Operador%20logico%20de%20transaccion.", ty: "Listed", subtype: "op_Logic_Tran", unique: false },
    ],
};

pub static GATES: ClassSpec = ClassSpec {
    class: "GateS",
    title: "Verificar depósito",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "fallo" },
    ],
    properties: &[
        PropertySpec { name: "Storage", hint: "Storage%20a%20evaluar.", ty: "EntityUse", subtype: "Storage", unique: false },
        PropertySpec { name: "op", hint: "Operador%20logico%20del%20storage.", ty: "Listed", subtype: "op_Logic_Stor", unique: false },
    ],
};

pub static GATHER: ClassSpec = ClassSpec {
    class: "Gather",
    title: "Reunir",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Juntar" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20transacciones%20a%20sincronizar%20por%20grupo%20de%20reunion", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static GENERATE: ClassSpec = ClassSpec {
    class: "Generate",
    title: "Generar",
    inputs: &[],
    outputs: &[
        PortSpec { ioid: 1, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20Transacciones%20a%20generar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Media", hint: "Frecuencia%20media%20de%20arribos", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modificador", hint: "Desvio%20o%20modificador", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "OffSet", hint: "Tiempo%20de%20espera", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Prioridad", hint: "Prioridad%20de%20la%20transaccion%20generada", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static INITIASV: ClassSpec = ClassSpec {
    class: "InitiaSV",
    title: "Inicializar SaveValue",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "SaveValue", hint: "SaveValue%20a%20inicializar", ty: "EntityDef", subtype: "SaveValue", unique: true },
        PropertySpec { name: "Valor", hint: "Valor%20inicial%20de%20SaveValue", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static INITIALLS: ClassSpec = ClassSpec {
    class: "InitialLS",
    title: "Inicializar llave lógica",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "LogicSwich", hint: "LogicSwich%20a%20inicicializar", ty: "EntityDef", subtype: "LogicSwich", unique: true },
        PropertySpec { name: "Valor", hint: "Valor%20de%20inicio", ty: "Listed", subtype: "op_Swich", unique: false },
    ],
};

pub static JOINME: ClassSpec = ClassSpec {
    class: "JoinMe",
    title: "Unirse a grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Grupo", hint: "Grupo%20num%E9rico%20al%20que%20se%20agrega%20un%20n%FAmero", ty: "EntityUse", subtype: "TransacGroup", unique: false },
    ],
};

pub static JOINNUMBER: ClassSpec = ClassSpec {
    class: "JoinNumber",
    title: "Agregar número a grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Grupo", hint: "Grupo%20num%E9rico%20al%20que%20se%20agrega%20un%20n%FAmero", ty: "EntityUse", subtype: "NumericGroup", unique: false },
        PropertySpec { name: "Numero", hint: "N%FAmero%20agregardo%20al%20grupo", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static LEAVE: ClassSpec = ClassSpec {
    class: "Leave",
    title: "Egresar de depósito",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20egresos", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Storage", hint: "Storage%20del%20cual%20egresa%20la%20transacci%F3n", ty: "EntityUse", subtype: "Storage", unique: false },
    ],
};

pub static LINK: ClassSpec = ClassSpec {
    class: "Link",
    title: "Encadenar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cadena", hint: "Cadena%20a%20la%20cual%20se%20enlazar%E1%20la%20transacci%F3n%20activa", ty: "EntityUse", subtype: "UserChain", unique: false },
        PropertySpec { name: "Modo", hint: "Modo%20en%20que%20se%20ordenaran%20las%20transacciones", ty: "Listed", subtype: "LinkMode", unique: false },
        PropertySpec { name: "Parametro", hint: "Parametro%20a%20ordenar%2C%20en%20caso%20de%20que%20el%20modo%20sea%20por%20Par%E1metro", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static LOGIC: ClassSpec = ClassSpec {
    class: "Logic",
    title: "Configurar llave lógica",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "LogicSwich", hint: "Llave%20l%F3gica%20a%20configurar", ty: "EntityUse", subtype: "LogicSwich", unique: false },
        PropertySpec { name: "Op", hint: "Operador%20de%20configuracci%F3n%20de%20la%20llave%20l%F3gica", ty: "Listed", subtype: "op_Swich", unique: false },
    ],
};

pub static LOOP: ClassSpec = ClassSpec {
    class: "Loop",
    title: "Repetir (loop)",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "loop" },
    ],
    properties: &[
        PropertySpec { name: "Parametro", hint: "Parametro%20a%20decrementar%20con%20cada%20iteracion", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static MARK: ClassSpec = ClassSpec {
    class: "Mark",
    title: "Marcar hora",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Parametro", hint: "Parametro%20donde%20se%20guarda%20la%20hora", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static MATCH: ClassSpec = ClassSpec {
    class: "Match",
    title: "Sincronizar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "RotuloMatch", hint: "R%F3tulo%20con%20el%20cual%20sincronizar", ty: "EntityUse", subtype: "Label", unique: false },
        PropertySpec { name: "RotuloPropio", hint: "R%F3tulo%20propio.", ty: "EntityDef", subtype: "Label", unique: true },
    ],
};

pub static MULTIFURCACION: ClassSpec = ClassSpec {
    class: "Multifurcacion",
    title: "Multifurcación",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada", caption: "Multifurcacion" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "%5BProb1%5D" },
        PortSpec { ioid: 3, name: "SalidaProb2", caption: "%5BProb2%5D" },
        PortSpec { ioid: 4, name: "SalidaProb3", caption: "%5BProb3%5D" },
        PortSpec { ioid: 5, name: "SalidaProb4", caption: "%5BProb4%5D" },
        PortSpec { ioid: 6, name: "SalidaProb5", caption: "%5BProb5%5D" },
    ],
    properties: &[
        PropertySpec { name: "MultifurcacionCH", hint: "", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "MultifurcacionGO", hint: "", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "Prob1", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob2", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob3", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob4", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob5", hint: "", ty: "FloatPercent", subtype: "", unique: false },
    ],
};

pub static MULTIFURCACIONP: ClassSpec = ClassSpec {
    class: "MultifurcacionP",
    title: "Multifurcación por parámetro",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada", caption: "Multifurcacion" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "%5BParametro%5D%3D1" },
        PortSpec { ioid: 3, name: "SalidaProb2", caption: "%5BParametro%5D%3D2" },
        PortSpec { ioid: 4, name: "SalidaProb3", caption: "%5BParametro%5D%3D3" },
        PortSpec { ioid: 5, name: "SalidaProb4", caption: "%5BParametro%5D%3D4" },
        PortSpec { ioid: 6, name: "SalidaProb5", caption: "%5BParametro%5D%3D5" },
    ],
    properties: &[
        PropertySpec { name: "MultifurcacionGO", hint: "", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "Parametro", hint: "", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "Prob1", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob2", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob3", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob4", hint: "", ty: "FloatPercent", subtype: "", unique: false },
        PropertySpec { name: "Prob5", hint: "", ty: "FloatPercent", subtype: "", unique: false },
    ],
};

pub static NOTE: ClassSpec = ClassSpec {
    class: "Note",
    title: "Nota",
    inputs: &[],
    outputs: &[],
    properties: &[],
};

pub static PREEMPT: ClassSpec = ClassSpec {
    class: "Preempt",
    title: "Interrumpir recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "SalidaAlt", caption: "Propietaria" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20interrumpir%20por%20la%20transacci%F3n%20activa.", ty: "EntityUse", subtype: "Facility", unique: false },
        PropertySpec { name: "ModoArrebato", hint: "Modo%20de%20arrebato%20temporal%20o%20permanete.", ty: "Listed", subtype: "RemoveMode", unique: false },
        PropertySpec { name: "ModoInterrupcion", hint: "Modo%20de%20interrupci%F3n%20o%20por%20prioridad", ty: "Listed", subtype: "PriorityMode", unique: false },
        PropertySpec { name: "Remanente", hint: "Par%E1metro%20de%20la%20transacci%F3n%20propietaria%20donde%20se%20guarda%20el%20remanete", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static PRIORITY: ClassSpec = ClassSpec {
    class: "Priority",
    title: "Asignar prioridad",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Prioridad", hint: "N%BA%20prioridad%20de%20la%20transacci%F3n.%20Cuanto%20mas%20grande%20sea%20el%20n%BA%2C%20mayor%20es%20la%20prioridad.", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static QTABLE: ClassSpec = ClassSpec {
    class: "QTable",
    title: "Definir tabla de cola",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "cantidad%20de%20clases%20de%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
        PropertySpec { name: "NombreTabla", hint: "Nombre%20de%20la%20tabla%20a%20definir", ty: "EntityDef", subtype: "QTable", unique: true },
        PropertySpec { name: "PrimerIntervalo", hint: "Limite%20superior%20de%20la%20primer%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
        PropertySpec { name: "Queue", hint: "Queue%20de%20Entrada1%20a%20la%20tabla", ty: "EntityDef", subtype: "Queue", unique: false },
        PropertySpec { name: "TamFrecuencia", hint: "Limite%20superior%20de%20la%20primer%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static QUEUE: ClassSpec = ClassSpec {
    class: "Queue",
    title: "Entrar en cola",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20ingresos%20en%20la%20cola", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Queue", hint: "Queue%20en%20la%20que%20ingresa%20la%20transacci%F3n", ty: "EntityUse", subtype: "Queue", unique: false },
    ],
};

pub static RELEASE: ClassSpec = ClassSpec {
    class: "Release",
    title: "Liberar recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20liberar", ty: "EntityUse", subtype: "Facility", unique: false },
    ],
};

pub static REMOVEME: ClassSpec = ClassSpec {
    class: "RemoveMe",
    title: "Remover del grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20en%20%5BGrupo%5D" },
    ],
    properties: &[
        PropertySpec { name: "Grupo", hint: "Grupo%20num%E9rico%20al%20que%20se%20agrega%20un%20n%FAmero", ty: "EntityUse", subtype: "TransacGroup", unique: false },
    ],
};

pub static REMOVETRANS: ClassSpec = ClassSpec {
    class: "RemoveTrans",
    title: "Remover transacciones",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20rmv" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20transacciones%20a%20remover", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "CompOp", hint: "Operador%20relacional%3A%20CompParam%20OP%20CompValue", ty: "Listed", subtype: "op_RelacMaxMin", unique: false },
        PropertySpec { name: "CompOpcion", hint: "Que%20se%20compara", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "CompParam", hint: "Parametro%20a%20comparar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "CompValue", hint: "Valor%20a%20comparar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Grupo", hint: "Grupo%20num%E9rico%20al%20que%20se%20agrega%20un%20n%FAmero", ty: "EntityUse", subtype: "TransacGroup", unique: false },
        PropertySpec { name: "Modo", hint: "Cuantas%20se%20remueven", ty: "Listed", subtype: "OperationMode", unique: false },
    ],
};

pub static RETURN: ClassSpec = ClassSpec {
    class: "Return",
    title: "Devolver recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20liberar", ty: "EntityUse", subtype: "Facility", unique: false },
    ],
};

pub static SAVEVALUE: ClassSpec = ClassSpec {
    class: "SaveValue",
    title: "Asignar SaveValue",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Op", hint: "Operador", ty: "Listed", subtype: "op_AddSub", unique: false },
        PropertySpec { name: "SaveValue", hint: "SaveValue%20a%20assignar%20un%20valor", ty: "EntityUse", subtype: "SaveValue", unique: false },
        PropertySpec { name: "Valor", hint: "Valor%20a%20assignar%20o%20modificar%20al%20SaveValue", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static SCAN: ClassSpec = ClassSpec {
    class: "Scan",
    title: "Escanear grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "Escane%F3" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20escane%F3" },
    ],
    properties: &[
        PropertySpec { name: "CompOp", hint: "Operador%20condicional", ty: "Listed", subtype: "op_RelacMaxMin", unique: false },
        PropertySpec { name: "CompOpcion", hint: "Que%20se%20compara", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "CompParam", hint: "Parametro%20de%20la%20transacciones%20del%20Grupo%20a%20comparar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "CompValue", hint: "Valor%20a%20comparar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "DeOpción", hint: "Opci%F3n%20de%20extracci%F3n", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "DeParam", hint: "Parametro%20de%20la%20trancci%F3n%20del%20gurpo%2C%20del%20cual%20se%20extrae%20el%20valor", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "EnOpcion", hint: "Opci%F3n%20de%20copia", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "EnParam", hint: "Parametro%20al%20cual%20se%20copia%20el%20valor", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "Grupo", hint: "Grupo%20a%20escanera", ty: "EntityUse", subtype: "TransacGroup", unique: false },
    ],
};

pub static SEIZE: ClassSpec = ClassSpec {
    class: "Seize",
    title: "Tomar recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Facility", hint: "Facility%20a%20tomar", ty: "EntityUse", subtype: "Facility", unique: false },
    ],
};

pub static SELECT: ClassSpec = ClassSpec {
    class: "Select",
    title: "Seleccionar de grupo",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "ClaseSNA", hint: "Clase%20de%20SNA%20de%20entidad%20por%20el%20cual%20se%20realiza%20la%20seleccion", ty: "Listed", subtype: "entitySNAclass", unique: false },
        PropertySpec { name: "Param", hint: "par%E1metro%20donde%20se%20guarda%20el%20valor%20seleccionado", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "maximo", hint: "Maximo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "minimo", hint: "Minimo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "op", hint: "Operador%20M%E1ximo%20o%20Minimo.", ty: "Listed", subtype: "op_MaxMin", unique: false },
    ],
};

pub static SELECTF: ClassSpec = ClassSpec {
    class: "SelectF",
    title: "Seleccionar recurso",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20selec" },
    ],
    properties: &[
        PropertySpec { name: "Param", hint: "par%E1metro%20donde%20se%20guarda%20el%20valor%20seleccionado", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "maximo", hint: "Maximo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "minimo", hint: "Minimo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "op", hint: "Operador%20Logico%20de%20Facility.", ty: "Listed", subtype: "op_Logic_Faci", unique: false },
    ],
};

pub static SELECTL: ClassSpec = ClassSpec {
    class: "SelectL",
    title: "Seleccionar llave lógica",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20selec" },
    ],
    properties: &[
        PropertySpec { name: "Param", hint: "par%E1metro%20donde%20se%20guarda%20el%20valor%20seleccionado", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "maximo", hint: "Maximo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "minimo", hint: "Minimo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "op", hint: "Operador%20Logico%20de%20LogicSwich.", ty: "Listed", subtype: "op_Logic_Swich", unique: false },
    ],
};

pub static SELECTS: ClassSpec = ClassSpec {
    class: "SelectS",
    title: "Seleccionar depósito",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20selec" },
    ],
    properties: &[
        PropertySpec { name: "Param", hint: "par%E1metro%20donde%20se%20guarda%20el%20valor%20seleccionado", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "maximo", hint: "Maximo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "minimo", hint: "Minimo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "op", hint: "Operador%20Logico%20de%20Storage.", ty: "Listed", subtype: "op_Logic_Stor", unique: false },
    ],
};

pub static SELECTSNA: ClassSpec = ClassSpec {
    class: "SelectSNA",
    title: "Seleccionar entidad",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "No%20selec" },
    ],
    properties: &[
        PropertySpec { name: "ClaseSNA", hint: "Clase%20de%20SNA%20de%20entidad%20por%20el%20cual%20se%20realiza%20la%20seleccion", ty: "Listed", subtype: "entitySNAclass", unique: false },
        PropertySpec { name: "Param", hint: "Nombre%2C%20numero%20o%20%2Aparam%20donde%20se%20guarda%20el%20valor%20seleccionado", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "maximo", hint: "Maximo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "minimo", hint: "Minimo%20valor%20de%20la%20selecci%F3n%20%28Cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "op", hint: "Operador%20Relacional.", ty: "Listed", subtype: "op_Relac", unique: false },
        PropertySpec { name: "valorComp", hint: "Valor%20a%20comparar.%20%28cualquier%20SNA%29.", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static SPLIT: ClassSpec = ClassSpec {
    class: "Split",
    title: "Clonar transacción",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "AClonar" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "Original" },
        PortSpec { ioid: 3, name: "Salida2", caption: "Copias" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20Copias", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Param", hint: "Par%E1etro%20Donde%20se%20numeran%20las%20transacciones", ty: "EntityUse", subtype: "Parameter", unique: false },
    ],
};

pub static STORAGEDEF: ClassSpec = ClassSpec {
    class: "StorageDef",
    title: "Definir depósito",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Storage", hint: "Storage%20a%20Declarar", ty: "EntityDef", subtype: "Storage", unique: true },
        PropertySpec { name: "Tamaño", hint: "Capacidad%20del%20Storage", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static STORAGEUSE: ClassSpec = ClassSpec {
    class: "StorageUse",
    title: "Usar depósito",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20ingresos%2FEgresos%20del%20storage", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Media", hint: "Tiempo%20Medio%20de%20uso%20del%20storage", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modificador", hint: "Desvio%20o%20%20modificador%20de%20la%20media", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Queue", hint: "", ty: "EntityUse", subtype: "Queue", unique: false },
        PropertySpec { name: "Storage", hint: "Storage%20a%20utilizar%20por%20un%20tiempo", ty: "EntityUse", subtype: "Storage", unique: false },
    ],
};

pub static TABLE: ClassSpec = ClassSpec {
    class: "Table",
    title: "Definir tabla",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Atributo", hint: "Atributo%20de%20Entrada1%20a%20la%20tabla", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Cantidad", hint: "cantidad%20de%20clases%20de%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
        PropertySpec { name: "NombreTabla", hint: "Nombre%20de%20la%20tabla%20a%20definir", ty: "EntityDef", subtype: "Table", unique: true },
        PropertySpec { name: "PrimerIntervalo", hint: "Limite%20superior%20de%20la%20primer%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
        PropertySpec { name: "TamFrecuencia", hint: "Limite%20superior%20de%20la%20primer%20frecuencia", ty: "PosInteger", subtype: "", unique: false },
    ],
};

pub static TABULATE: ClassSpec = ClassSpec {
    class: "Tabulate",
    title: "Tabular",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
    ],
    properties: &[
        PropertySpec { name: "NombreTabla", hint: "Nombre%20de%20la%20tabla%20a%20definir", ty: "EntityUse", subtype: "Table", unique: false },
        PropertySpec { name: "Peso", hint: "Factor%20de%20peso", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static TERMINATE: ClassSpec = ClassSpec {
    class: "Terminate",
    title: "Terminar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Unidades", hint: "Cantidad%20de%20unidades%20a%20decrementar%20del%20START%2C%20por%20cada%20transacci%F3n%20que%20llega%20al%20m%F3dulo", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static TEST: ClassSpec = ClassSpec {
    class: "Test",
    title: "Comparar (test)",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Entrada1" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "Verdadero" },
        PortSpec { ioid: 3, name: "Salida2", caption: "Falso" },
    ],
    properties: &[
        PropertySpec { name: "1_Valor", hint: "Primer%20Valor", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "2_OP", hint: "Operador%20relacional", ty: "Listed", subtype: "op_Relac", unique: false },
        PropertySpec { name: "3_Valor", hint: "Valor%20a%20comparar%20con%20el%20primero", ty: "TSNA", subtype: "SNA", unique: false },
    ],
};

pub static TRANSFER: ClassSpec = ClassSpec {
    class: "Transfer",
    title: "Transferir",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "" },
        PortSpec { ioid: 3, name: "Salida2", caption: "resto" },
    ],
    properties: &[
        PropertySpec { name: "Prob", hint: "Probabilidad%20de%20continuar%20por%20la%20Salida1%201", ty: "", subtype: "MultiType", unique: false },
    ],
};

pub static UNLINK: ClassSpec = ClassSpec {
    class: "Unlink",
    title: "Desencadenar",
    inputs: &[
        PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" },
    ],
    outputs: &[
        PortSpec { ioid: 2, name: "Salida1", caption: "Desencaden%F3" },
        PortSpec { ioid: 3, name: "SalidaAlt", caption: "Desencadenadas" },
        PortSpec { ioid: 4, name: "Salida2", caption: "No%20desencaden%F3" },
    ],
    properties: &[
        PropertySpec { name: "Cadena", hint: "Cadena%20de%20la%20cual%20se%20desencadenaran%20transacciones", ty: "EntityUse", subtype: "UserChain", unique: false },
        PropertySpec { name: "Cantidad", hint: "Cantidad%20de%20transacciones%20a%20desencadenar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "CompOp", hint: "Operador%20de%20comparaci%F3n", ty: "Listed", subtype: "op_RelacMaxMin", unique: false },
        PropertySpec { name: "CompOpcion", hint: "Que%20se%20atributo%20comparo%20de%20la%20transacci%F3n%20encadenada", ty: "Listed", subtype: "PriorityOption", unique: false },
        PropertySpec { name: "CompParam", hint: "Parametro%20a%20comparar", ty: "EntityUse", subtype: "Parameter", unique: false },
        PropertySpec { name: "CompValue", hint: "Valor%20a%20comparar", ty: "TSNA", subtype: "SNA", unique: false },
        PropertySpec { name: "Modo", hint: "Como%20se%20alteran", ty: "Listed", subtype: "OperationMode", unique: false },
    ],
};

pub static VARIABLE: ClassSpec = ClassSpec {
    class: "Variable",
    title: "Definir variable",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Expresion", hint: "Expresion%20matematica", ty: "Expresion", subtype: "Expresion", unique: false },
        PropertySpec { name: "NombreVar", hint: "Nombre%20o%20Nro%20de%20la%20variable", ty: "EntityDef", subtype: "Variable", unique: true },
    ],
};

pub static VARIABLEN: ClassSpec = ClassSpec {
    class: "VariableN",
    title: "Definir variable normal",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Desvio", hint: "Sigma", ty: "Expresion", subtype: "Expresion", unique: false },
        PropertySpec { name: "Media", hint: "Valor%20medio", ty: "Expresion", subtype: "Expresion", unique: false },
        PropertySpec { name: "NombreFunc", hint: "Nombre%20de%20la%20Funci%F3n", ty: "EntityDef", subtype: "Function", unique: true },
        PropertySpec { name: "NombreVar", hint: "Nombre%20o%20Nro%20de%20la%20variable", ty: "EntityDef", subtype: "Variable", unique: true },
        PropertySpec { name: "Seed", hint: "Semilla", ty: "EntityUse", subtype: "RandomSeed", unique: false },
    ],
};

pub static VARIABLEA: ClassSpec = ClassSpec {
    class: "VariableA",
    title: "Definir variable aleatoria",
    inputs: &[],
    outputs: &[],
    properties: &[
        PropertySpec { name: "Maximo", hint: "Maximo", ty: "Expresion", subtype: "Expresion", unique: false },
        PropertySpec { name: "Minimo", hint: "Minimo", ty: "Expresion", subtype: "Expresion", unique: false },
        PropertySpec { name: "NombreVar", hint: "Nombre%20o%20Nro%20de%20la%20variable", ty: "EntityDef", subtype: "Variable", unique: true },
        PropertySpec { name: "Seed", hint: "Seed", ty: "EntityUse", subtype: "RandomSeed", unique: false },
    ],
};

