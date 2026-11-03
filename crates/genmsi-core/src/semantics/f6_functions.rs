//! The contracts of the family: the functions and the variables.
//!
//! The classes of this family are the ones that give a model its numbers:
//! the functions that turn the value of an attribute into another value, the
//! variables that hold an equation, the table that collects the values, and
//! the multifurcations that split the traffic of a transaction among its
//! branches.
//!
//! The text says what the engine does. The engine is measured against the
//! original (`fixtures/outputs/`), and the divergences that are left are
//! pinned by the tests of `crates/genmsi-core/tests/diferencias_*.rs`.

use super::contract::Semantics;

/// The contracts of the family, in the order of the catalogue.
pub static CONTRACTS: &[(&str, Semantics)] = &[
    (
        "Function",
        Semantics {
            flow: "It does not touch the flow of any transaction: the class creates no block, so no transaction enters it and none leaves it. What it declares is the function `Function.NombreFunc`, that the rest of the model reads with `FN:<name>`.",
            blocking: "It blocks nothing: reading a function evaluates it at once and the transaction goes on.",
            resources: "It takes nothing and gives nothing back: a function is a definition, not a resource, and it outlives every transaction.",
            gotchas: "The function takes no argument at the call site: the independent variable is `Function.Atributo` (the default `P:Param`), and it is evaluated against the transaction that reads the function, so the same `FN:<name>` gives a different value to every transaction that reads it with a parameter of its own. `Function.Tipo` decides how the table of `Function.Tabla` is read: `Discreta` takes the value of the last point whose x is not greater than the argument (the value of the first point when the argument is below them all), `Continua` interpolates between the points and holds the first and the last value outside them, and `Eval` takes the value of the last point whose x is not greater than the argument and gives 0 when the argument is below them all.",
        },
    ),
    (
        "FunctionDist",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares the distribution function `FunctionDist.NombreDist`, read with `FN:<name>`, that gives a whole number between 1 and the number of probabilities of `FunctionDist.Distribucion`.",
            blocking: "It blocks nothing: the draw is made at once, when the function is read.",
            resources: "It takes nothing and gives nothing back; what it consumes is the random stream `FunctionDist.Semilla`, one number per read.",
            gotchas: "The value is the number of the result, not a probability: `FunctionDist.Distribucion` is the ordered list of the probabilities of the results 1, 2, ..., n, and the engine normalizes it, so the list need not add up to 100. Every read is a new draw, so a function read twice in the same expression gives two different numbers.",
        },
    ),
    (
        "FunctionExpon",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares the function `FunctionExpon.NombreFunc`, read with `FN:<name>`, whose value is an exponential deviate of mean 1 drawn from the random stream `FunctionExpon.Seed`.",
            blocking: "It blocks nothing: the draw is made at once, when the function is read.",
            resources: "It takes nothing and gives nothing back; what it consumes is the random stream `FunctionExpon.Seed`, one number per read.",
            gotchas: "The deviate has mean 1, not the mean of the model: the class is the multiplier of a delay (`Generate.Modificador` = `FN:<name>` multiplies `Generate.Media` by it and truncates the product to a whole number). The listing writes it the way the original does, `FUNCTION <name> RN<seed>,C24` with the 24 points of the exponential.",
        },
    ),
    (
        "MFunction",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares one function per name of `MFunction.NombresFunc`, read with `FN:<name>`, and all of them share the same independent variable, `MFunction.Atributo`.",
            blocking: "It blocks nothing: reading a function evaluates it at once and the transaction goes on.",
            resources: "It takes nothing and gives nothing back: the functions are definitions and they outlive every transaction.",
            gotchas: "The row of `MFunction.Matriz` that belongs to a function holds its outputs, not its inputs: the value of the function for the input 1 is the first element of the row, for the input 2 the second, and so on. The listing writes one `E` function per name with the table of its row (`FuncA FUNCTION P$Param,E3` with `1,11/2,12/3,13`), the same as the original; a name of the list without a row of the matrix declares no function.",
        },
    ),
    (
        "MFunctionDist",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares one distribution function per name of `MFunctionDist.NombresDist`, read with `FN:<name>`, and each of them gives a whole number between 1 and the number of probabilities of its own row.",
            blocking: "It blocks nothing: the draw is made at once, when the function is read.",
            resources: "It takes nothing and gives nothing back; what it consumes is the random stream `MFunctionDist.Semilla`, one number per read, the same stream for every function of the class.",
            gotchas: "The row of `MFunctionDist.Resultados` that belongs to a function is the ordered list of the probabilities of its results 1, 2, ..., n, and the engine normalizes it, so the row need not add up to 100. The listing writes one `D` function per name with the running sum of the probabilities of its row (`Func1 FUNCTION RN4,D3` with `0.2,1/0.5,2/1,3`), the same as the original.",
        },
    ),
    (
        "Multifurcacion",
        Semantics {
            flow: "The transaction goes on by the output of the branch it takes: the shares `Multifurcacion.Prob1`..`Prob5` say how much of the traffic each branch takes (the first share goes to `Salida1`, the second to `SalidaProb2`, and so on) and the draw is made at every arrival. When the model names them, the class also declares the two functions of the original: the distribution `Multifurcacion.MultifurcacionCH` and the destination function `Multifurcacion.MultifurcacionGO`.",
            blocking: "It blocks nothing: the branch is taken at once.",
            resources: "It takes nothing and gives nothing back: the transaction keeps whatever it had on the way it takes the branch.",
            gotchas: "The shares are relative: the engine normalizes them by their sum, so they need not add up to 100, and it reads `20%` and `20` as the same share; the branch is drawn on the random stream of the block itself, not on the `MultifurcacionCH` function that the listing writes. A `Multifurcacion` with no share at all does not compile. The `LOC` column of the report is fifteen wide and a name that does not fit eats the space that separates it from the `RETRY` column: the row of the multifurcation reads `0MultifurcacionP` and not `0 MultifurcacionP`, the way the original writes it (`crates/genmsi-core/tests/diferencias_restantes.rs`).",
        },
    ),
    (
        "MultifurcacionP",
        Semantics {
            flow: "The branch is the value of the parameter `MultifurcacionP.Parametro`: the value 1 takes `Salida1`, 2 takes `SalidaProb2`, and so on, and the block writes the number of the branch it took back in that parameter. When the model names it, the class declares the destination function `MultifurcacionP.MultifurcacionGO`, that the listing writes as `FUNCTION P$<Parametro>,D<n>` with the table of the branches.",
            blocking: "It blocks nothing: the branch is taken at once.",
            resources: "It takes nothing and gives nothing back; the only thing it writes is in the parameter of the transaction, which is its own.",
            gotchas: "A transaction that arrives with no value, or with a value outside the branches, takes the first branch. The shares `Prob1`..`Prob5` that the catalogue lists for the class are ignored: the class of the original has no such properties, it routes by the parameter alone. The `LOC` column of the report is fifteen wide and a name that does not fit eats the space that separates it from the `RETRY` column: the row of the multifurcation reads `0MultifurcacionP` and not `0 MultifurcacionP`, the way the original writes it (`crates/genmsi-core/tests/diferencias_restantes.rs`).",
        },
    ),
    (
        "Table",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares the table `Table.NombreTabla`, whose classes go from `Table.PrimerIntervalo`, in steps of `Table.TamFrecuencia`, and are `Table.Cantidad` of them. The recording is done by the `Tabulate` class, which counts the value of `Table.Atributo` of the transaction that passes it.",
            blocking: "It blocks nothing: declaring a table waits for nothing.",
            resources: "It takes nothing and gives nothing back: the table belongs to the run and outlives every transaction, and the values it holds are the ones the `Tabulate` blocks record.",
            gotchas: "The report writes the tables the other way round than the model declares them (the last table declared is the first one of the report) while the listing keeps the order of the declaration; the original writes it the same way (`crates/genmsi-core/tests/diferencias_restantes.rs`). `Table.Atributo` is required: a table without it does not compile.",
        },
    ),
    (
        "Variable",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares the variable `Variable.NombreVar`, read with `V:<name>`, whose value is the result of `Variable.Expresion` evaluated against the transaction that reads it.",
            blocking: "It blocks nothing: the equation is evaluated at once, when the variable is read.",
            resources: "It takes nothing and gives nothing back: the variable holds no state, it is a definition and it outlives every transaction.",
            gotchas: "The variable is not a number stored somewhere: its equation is evaluated again on every read, so two reads of the same variable in the same expression can give two different values, and the `P:<parameter>` of the equation is the one of the transaction that reads it, not of one fixed transaction.",
        },
    ),
    (
        "VariableA",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares the random variable `VariableA.NombreVar`, read with `V:<name>`, whose value is a whole number between `VariableA.Minimo` and `VariableA.Maximo`.",
            blocking: "It blocks nothing: the draw is made at once, when the variable is read.",
            resources: "It takes nothing and gives nothing back; what it consumes is the random stream `VariableA.Seed`, one number per read.",
            gotchas: "Every read is a new draw: the variable does not keep the number it gave the first time. The bounds are rounded to whole numbers, and the listing writes the equation of the original (`Azar VARIABLE ((RN80#1000)+RN80)@(1+(10)-(1))+(1)`).",
        },
    ),
    (
        "VariableN",
        Semantics {
            flow: "It creates no block, so no transaction enters it: the class declares two things, the function `VariableN.NombreFunc` (the table of the normal curve, of the type `C`, drawn from the random stream `VariableN.Seed`) and the variable `VariableN.NombreVar`, read with `V:<name>`, whose value is `VariableN.Media` plus `VariableN.Desvio` times the deviate of that function.",
            blocking: "It blocks nothing: the draw is made at once, when the variable is read.",
            resources: "It takes nothing and gives nothing back; what it consumes is the random stream `VariableN.Seed`, one number per read.",
            gotchas: "The table of the function holds ten times the quantile of the normal curve, which is why the equation the engine writes divides by ten (`60+(1#FN$NORMAL)/10`). The class declares two entities and the original hides the name of the function (`HIDE NombreFunc`): rename it and the variable follows the new name.",
        },
    ),
];
