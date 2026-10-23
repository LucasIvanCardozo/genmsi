//! pest-driven parser for the expression language.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use super::{
    BinOp, Expr, NameSource, ProgramExpr, RefKind, SystemSymbol,
};
use crate::error::EngineError;

#[derive(Parser)]
#[grammar = "expr/grammar.pest"]
struct ExprParser;

/// Parse expression text into a [`ProgramExpr`].
pub fn parse(source: &str) -> Result<ProgramExpr, EngineError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(EngineError::Expression {
            text: source.to_string(),
            message: "empty expression".to_string(),
        });
    }
    let mut pairs = ExprParser::parse(Rule::expr, trimmed).map_err(|err| EngineError::Expression {
        text: source.to_string(),
        message: err.to_string(),
    })?;
    let root = pairs.next().ok_or_else(|| EngineError::Expression {
        text: source.to_string(),
        message: "nothing parsed".to_string(),
    })?;
    let expr = build(root).map_err(|message| EngineError::Expression {
        text: source.to_string(),
        message,
    })?;
    Ok(ProgramExpr::new(trimmed.to_string(), expr))
}

fn build(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    match pair.as_rule() {
        Rule::sum => build_sum(pair),
        Rule::product => build_product(pair),
        Rule::unary => build_unary(pair),
        Rule::neg => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| "unary minus without operand".to_string())?;
            Ok(Expr::Neg(Box::new(build(inner)?)))
        }
        Rule::number => pair
            .as_str()
            .parse::<f64>()
            .map(Expr::Num)
            .map_err(|_| format!("invalid number {:?}", pair.as_str())),
        Rule::placeholder => Ok(Expr::Placeholder(pair.as_str().trim_matches(['[', ']']).to_string())),
        Rule::percent => {
            let text = pair.as_str().trim_end_matches('%');
            text.parse::<f64>()
                .map(|value| Expr::Num(value * 10.0))
                .map_err(|_| format!("invalid percentage {:?}", pair.as_str()))
        }
        Rule::typed => build_typed(pair),
        Rule::indirect => build_indirect(pair),
        Rule::system => build_system(pair.as_str()),
        Rule::bare => Ok(Expr::Ref {
            kind: RefKind::Parameter,
            name: NameSource::Literal(pair.as_str().to_string()),
        }),
        other => Err(format!("unexpected token {other:?}")),
    }
}

fn build_sum(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    let mut inner = pair.into_inner();
    let mut left = build(
        inner
            .next()
            .ok_or_else(|| "empty sum".to_string())?,
    )?;
    while let Some(op) = inner.next() {
        let right = inner
            .next()
            .ok_or_else(|| "missing operand after operator".to_string())?;
        let bin = match op.as_str() {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            other => return Err(format!("unexpected additive operator {other:?}")),
        };
        left = Expr::Bin(bin, Box::new(left), Box::new(build(right)?));
    }
    Ok(left)
}

fn build_product(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    let mut inner = pair.into_inner();
    let mut left = build(
        inner
            .next()
            .ok_or_else(|| "empty product".to_string())?,
    )?;
    while let Some(op) = inner.next() {
        let right = inner
            .next()
            .ok_or_else(|| "missing operand after operator".to_string())?;
        let bin = match op.as_str() {
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            // The original's expression editor has no `*` key: it offers `/`,
            // `%`, `#` and `@`. Its `#` is the multiplication, and its models
            // read that way ("15#paraLlevar" is a surcharge per delivery, not
            // a fraction).
            "#" => BinOp::Mul,
            "@" => BinOp::Mod,
            other => return Err(format!("unexpected multiplicative operator {other:?}")),
        };
        left = Expr::Bin(bin, Box::new(left), Box::new(build(right)?));
    }
    Ok(left)
}

fn build_unary(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| "empty unary expression".to_string())?;
    build(inner)
}

fn build_typed(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    let mut inner = pair.into_inner();
    let kind_text = inner
        .next()
        .ok_or_else(|| "typed reference without prefix".to_string())?;
    let kind = RefKind::from_prefix(kind_text.as_str())
        .ok_or_else(|| format!("unknown reference prefix {:?}", kind_text.as_str()))?;
    let source = inner
        .next()
        .ok_or_else(|| "typed reference without name".to_string())?;
    let name = name_source(source)?;
    Ok(Expr::Ref { kind, name })
}

fn build_indirect(pair: Pair<'_, Rule>) -> Result<Expr, String> {
    let source = pair
        .into_inner()
        .next()
        .ok_or_else(|| "indirect reference without name".to_string())?;
    let name = name_source(source)?;
    // A bare `*X` is the value of parameter X (GPSS indirection).
    match name {
        NameSource::Literal(name) | NameSource::Indirect(name) => Ok(Expr::Ref {
            kind: RefKind::Parameter,
            name: NameSource::Literal(name),
        }),
    }
}

fn name_source(pair: Pair<'_, Rule>) -> Result<NameSource, String> {
    match pair.as_rule() {
        Rule::name => Ok(NameSource::Literal(pair.as_str().trim().to_string())),
        Rule::indirect_name => {
            let name = pair
                .into_inner()
                .next()
                .ok_or_else(|| "indirect name without parameter".to_string())?;
            Ok(NameSource::Indirect(name.as_str().trim().to_string()))
        }
        other => Err(format!("unexpected name token {other:?}")),
    }
}

fn build_system(text: &str) -> Result<Expr, String> {
    match text {
        "AC1" => Ok(Expr::System(SystemSymbol::Clock)),
        "C1" => Ok(Expr::System(SystemSymbol::AbsoluteClock)),
        "TC1" => Ok(Expr::System(SystemSymbol::TerminationCounter)),
        other => {
            if let Some(stream) = other.strip_prefix("RN") {
                let stream = stream
                    .parse::<u32>()
                    .map_err(|_| format!("invalid random stream {other:?}"))?;
                Ok(Expr::System(SystemSymbol::Random(stream)))
            } else {
                Err(format!("unknown system symbol {other:?}"))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_reference_kinds_used_by_the_models() {
        for source in [
            "P:ImpacienciaMedia",
            "P$pValue",
            "X:IdPescadorMayorPeso",
            "X$paradescongelar",
            "V:cantCajas",
            "V$faltantes",
            "FN:MediaClientes",
            "F:Expon",
            "Q:qpostvacunacion",
            "S:SalaEspera",
            "R:Recepcion",
            "CH:Cadena",
            "CH:*NroColectivo",
            "N$A_Queue",
            "W$A_Queue",
            "BV:flag",
            "MX:m",
        ] {
            parse(source).unwrap_or_else(|err| panic!("{source}: {err}"));
        }
    }

    #[test]
    fn parses_complex_model_expressions() {
        for source in [
            "(1+((X:paradescongelar-X:descongeladas)@5))#5",
            "P:vacunasdesc+X:descongeladas-X:paradescongelar",
            "FN:FuncMediaCocina+P:paraLlevar#30",
            "1+P:Parada@10",
            "160+(P:Tipo-1)#30",
            "1+(3)-(1)",
        ] {
            parse(source).unwrap_or_else(|err| panic!("{source}: {err}"));
        }
    }

    #[test]
    fn unknown_prefix_is_reported_with_the_source() {
        let err = parse("ZZ:Foo").expect_err("must fail");
        let text = err.to_string();
        assert!(text.contains("ZZ:Foo"), "{text}");
    }
}
