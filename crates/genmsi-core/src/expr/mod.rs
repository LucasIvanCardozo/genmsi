//! The GenMSI expression language: parsing and evaluation of SNAs.
//!
//! Expressions are parsed once while the model is compiled (decision D4: the
//! pest grammar is simultaneously parser, documentation and error source) and
//! evaluated many times against a [`Ctx`] supplied by the engine.

mod parse;

use std::fmt;

pub use parse::parse;

use crate::error::{EngineError, numeric_name};

/// Binary operators of the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `#` integer division
    IntDiv,
    /// `@` modulo
    Mod,
}

impl BinOp {
    fn apply(self, left: f64, right: f64) -> f64 {
        match self {
            Self::Add => left + right,
            Self::Sub => left - right,
            Self::Mul => left * right,
            Self::Div => {
                if right == 0.0 {
                    0.0
                } else {
                    left / right
                }
            }
            Self::IntDiv => {
                if right == 0.0 {
                    0.0
                } else {
                    (left / right).trunc()
                }
            }
            Self::Mod => {
                if right == 0.0 {
                    0.0
                } else {
                    left.rem_euclid(right)
                }
            }
        }
    }
}

/// Entity kind addressed by a typed reference (`P:`, `Q:`, `CH:` ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    /// `P:` transaction parameter.
    Parameter,
    /// `X:` savevalue.
    SaveValue,
    /// `V:` variable.
    Variable,
    /// `FN:` / `F:` function.
    Function,
    /// `Q:` queue content.
    Queue,
    /// `S:` storage units in use.
    Storage,
    /// `R:` facility busy flag.
    Facility,
    /// `CH:` user chain size.
    Chain,
    /// `LS:` / `L:` logic switch value.
    LogicSwitch,
    /// `N:` transactions that passed a label.
    Passed,
    /// `W:` transactions waiting at a label.
    Waiting,
    /// `BV:` boolean variable.
    BooleanVariable,
    /// `MX:` matrix cell.
    Matrix,
}

impl RefKind {
    /// Parse the one/two-letter prefix of a typed reference.
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        Some(match prefix {
            "P" => Self::Parameter,
            "X" => Self::SaveValue,
            "V" => Self::Variable,
            "FN" | "F" => Self::Function,
            "Q" => Self::Queue,
            "S" => Self::Storage,
            "R" => Self::Facility,
            "CH" => Self::Chain,
            "LS" | "L" => Self::LogicSwitch,
            "N" => Self::Passed,
            "W" => Self::Waiting,
            "BV" => Self::BooleanVariable,
            "MX" => Self::Matrix,
            _ => return None,
        })
    }
}

impl fmt::Display for RefKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Parameter => "P",
            Self::SaveValue => "X",
            Self::Variable => "V",
            Self::Function => "FN",
            Self::Queue => "Q",
            Self::Storage => "S",
            Self::Facility => "R",
            Self::Chain => "CH",
            Self::LogicSwitch => "LS",
            Self::Passed => "N",
            Self::Waiting => "W",
            Self::BooleanVariable => "BV",
            Self::Matrix => "MX",
        })
    }
}

/// How the name of a referenced entity is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameSource {
    /// A literal entity name.
    Literal(String),
    /// Indirect: the name is the numeric value of this parameter (GPSS `*P`).
    Indirect(String),
}

impl NameSource {
    /// Resolve the name through the context.
    pub fn resolve(&self, ctx: &mut dyn Ctx) -> String {
        match self {
            Self::Literal(name) => name.clone(),
            Self::Indirect(parameter) => numeric_name(ctx.parameter(parameter)),
        }
    }
}

impl fmt::Display for NameSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(name) => f.write_str(name),
            Self::Indirect(parameter) => write!(f, "*{parameter}"),
        }
    }
}

/// System symbols available without a prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemSymbol {
    /// `AC1`: model clock.
    Clock,
    /// `C1`: absolute clock.
    AbsoluteClock,
    /// `TC1`: remaining termination count.
    TerminationCounter,
    /// `RNn`: uniform draw from random stream `n`.
    Random(u32),
}

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal number.
    Num(f64),
    /// Unary minus.
    Neg(Box<Expr>),
    /// Binary operation.
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// Typed entity reference.
    Ref {
        /// Entity kind.
        kind: RefKind,
        /// Entity name.
        name: NameSource,
    },
    /// System symbol.
    System(SystemSymbol),
    /// A `[containerParameter]` placeholder that the compiler must resolve.
    Placeholder(String),
}

impl Expr {
    /// Whether the expression still contains an unresolved `[placeholder]`.
    pub fn placeholder(&self) -> Option<&str> {
        match self {
            Self::Placeholder(name) => Some(name),
            Self::Neg(inner) => inner.placeholder(),
            Self::Bin(_, left, right) => left.placeholder().or_else(|| right.placeholder()),
            _ => None,
        }
    }

    /// Evaluate against a simulation context.
    pub fn eval(&self, ctx: &mut dyn Ctx) -> f64 {
        match self {
            Self::Num(value) => *value,
            Self::Neg(inner) => -inner.eval(ctx),
            Self::Bin(op, left, right) => op.apply(left.eval(ctx), right.eval(ctx)),
            Self::System(symbol) => match symbol {
                SystemSymbol::Clock => ctx.clock(),
                SystemSymbol::AbsoluteClock => ctx.absolute_clock(),
                SystemSymbol::TerminationCounter => ctx.termination_counter(),
                SystemSymbol::Random(stream) => ctx.random(*stream),
            },
            Self::Ref { kind, name } => {
                let resolved = name.resolve(ctx);
                ctx.reference(*kind, &resolved)
            }
            Self::Placeholder(_) => 0.0,
        }
    }
}

/// Everything an expression may read from or write to.
///
/// Implemented by the simulation state. Unknown names follow GPSS autodefinition
/// rules (a new facility, storage, queue, chain or savevalue starts at zero).
pub trait Ctx {
    /// Current transaction parameter.
    fn parameter(&mut self, name: &str) -> f64;
    /// Value of a typed reference.
    fn reference(&mut self, kind: RefKind, name: &str) -> f64;
    /// Model clock (`AC1`).
    fn clock(&mut self) -> f64;
    /// Absolute clock (`C1`).
    fn absolute_clock(&mut self) -> f64;
    /// Remaining termination count (`TC1`).
    fn termination_counter(&mut self) -> f64;
    /// Uniform draw in `[0, 1)` from stream `stream`.
    fn random(&mut self, stream: u32) -> f64;
}

/// A parsed expression with its source text, ready for evaluation and display.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramExpr {
    source: String,
    expr: Expr,
}

impl ProgramExpr {
    pub(crate) fn new(source: String, expr: Expr) -> Self {
        Self { source, expr }
    }

    /// Parse expression text.
    pub fn parse(source: &str) -> Result<Self, EngineError> {
        parse(source)
    }

    /// Evaluate the expression.
    pub fn eval(&self, ctx: &mut dyn Ctx) -> f64 {
        self.expr.eval(ctx)
    }

    /// Source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Parsed tree, for tests and inspection.
    pub fn ast(&self) -> &Expr {
        &self.expr
    }
}

impl fmt::Display for ProgramExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

impl serde::Serialize for ProgramExpr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Fake {
        parameters: HashMap<String, f64>,
        savevalues: HashMap<String, f64>,
        random_draws: Vec<u32>,
        now: f64,
    }

    impl Ctx for Fake {
        fn parameter(&mut self, name: &str) -> f64 {
            *self.parameters.get(name).unwrap_or(&0.0)
        }

        fn reference(&mut self, kind: RefKind, name: &str) -> f64 {
            match kind {
                RefKind::Parameter => self.parameter(name),
                RefKind::SaveValue => *self.savevalues.get(name).unwrap_or(&0.0),
                RefKind::Storage => 2.0,
                RefKind::Queue => 3.0,
                _ => 0.0,
            }
        }

        fn clock(&mut self) -> f64 {
            self.now
        }

        fn absolute_clock(&mut self) -> f64 {
            self.now
        }

        fn termination_counter(&mut self) -> f64 {
            1.0
        }

        fn random(&mut self, stream: u32) -> f64 {
            self.random_draws.push(stream);
            0.5
        }
    }

    fn eval(source: &str) -> f64 {
        ProgramExpr::parse(source)
            .unwrap_or_else(|err| panic!("{source:?}: {err}"))
            .eval(&mut Fake::default())
    }

    #[test]
    fn arithmetic_follows_gpss_operators() {
        assert_eq!(eval("1+2*3"), 7.0);
        assert_eq!(eval("(1+2)*3"), 9.0);
        assert_eq!(eval("7#2"), 3.0);
        assert_eq!(eval("7@2"), 1.0);
        assert_eq!(eval("1.5+0.5"), 2.0);
        assert_eq!(eval("-3+1"), -2.0);
    }

    #[test]
    fn zero_division_is_zero_instead_of_panicking() {
        assert_eq!(eval("5/0"), 0.0);
        assert_eq!(eval("5#0"), 0.0);
    }

    #[test]
    fn parses_typed_references() {
        let expr = ProgramExpr::parse("P:Media").unwrap();
        assert_eq!(
            expr.ast(),
            &Expr::Ref {
                kind: RefKind::Parameter,
                name: NameSource::Literal("Media".into())
            }
        );
        assert_eq!(ProgramExpr::parse("FN:Horario").unwrap().ast(), &Expr::Ref { kind: RefKind::Function, name: NameSource::Literal("Horario".into()) });
        assert_eq!(ProgramExpr::parse("X$Count").unwrap().ast(), &Expr::Ref { kind: RefKind::SaveValue, name: NameSource::Literal("Count".into()) });
    }

    #[test]
    fn parses_indirect_names() {
        let expr = ProgramExpr::parse("CH:*NroColectivo").unwrap();
        assert_eq!(
            expr.ast(),
            &Expr::Ref {
                kind: RefKind::Chain,
                name: NameSource::Indirect("NroColectivo".into())
            }
        );
        let mut ctx = Fake::default();
        ctx.parameters.insert("NroColectivo".into(), 7.0);
        assert_eq!(NameSource::Indirect("NroColectivo".into()).resolve(&mut ctx), "7");
    }

    #[test]
    fn parses_system_symbols() {
        assert_eq!(
            ProgramExpr::parse("AC1").unwrap().ast(),
            &Expr::System(SystemSymbol::Clock)
        );
        assert_eq!(
            ProgramExpr::parse("TC1").unwrap().ast(),
            &Expr::System(SystemSymbol::TerminationCounter)
        );
        let expr = ProgramExpr::parse("RN80").unwrap();
        assert_eq!(expr.ast(), &Expr::System(SystemSymbol::Random(80)));
        let mut fake = Fake::default();
        assert_eq!(expr.eval(&mut fake), 0.5);
        assert_eq!(fake.random_draws, vec![80]);
    }

    #[test]
    fn exposes_unresolved_placeholders() {
        let expr = ProgramExpr::parse("[CantidadOficinas]").unwrap();
        assert_eq!(expr.ast().placeholder(), Some("CantidadOficinas"));
        let nested = ProgramExpr::parse("1+[Cantidad]*2").unwrap();
        assert_eq!(nested.ast().placeholder(), Some("Cantidad"));
        assert_eq!(ProgramExpr::parse("1+2").unwrap().ast().placeholder(), None);
    }

    #[test]
    fn rejects_garbage_with_the_source_in_the_message() {
        let err = ProgramExpr::parse("P:").expect_err("must fail");
        let text = err.to_string();
        assert!(text.contains("P:"), "{text}");
    }

    #[test]
    fn evaluates_references_through_the_context() {
        let mut ctx = Fake::default();
        ctx.parameters.insert("Media".into(), 4.0);
        ctx.savevalues.insert("Count".into(), 10.0);
        let expr = ProgramExpr::parse("P:Media*2+X:Count").unwrap();
        assert_eq!(expr.eval(&mut ctx), 18.0);
    }
}
