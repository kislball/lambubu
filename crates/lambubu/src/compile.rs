//! Compilation and preprocessing utilities for Lambubu.
//! It is responsible for parsing and preprocessing $\lambda$-expressions into [`crate::Term`].
//!
//!
//! # Lambda syntax
//! Expression is any of the following:
//! 1. A lambda term `(\X.Y)`, where `X` is any variable name(lowercase) and `Y` is any expression
//! 2. A variable `a`, where `a` is a sequence of lowercase characters
//! 3. A macro `A`, where `A` is a sequence of uppercase characters
//! 4. Application `(A B)`, where `A` is an expression and `B` is one or more expressions.
//! 5. Another expression in parenthesis
//!
//! Input may also contain macro definitions: `MACRO_NAME::EXPRESSION`
//!
//! Each line contains at most one definition or expression
//!
//! # Example
//! ```rust
//! use lambubu::{compile_term, CompoundEnvironment, Term};
//! let term = compile_term("(\\x.x y)", &mut CompoundEnvironment::new(vec![])).unwrap();
//!
//! assert_eq!(term, Term::app(Term::abs("x", Term::var("x")), Term::var("y")));
//! ```
//!
//! ```rust
//! use lambubu::{compile_term, CompoundEnvironment, Term, compile::CompilationError};
//! let term = compile_term("(\\x.x y", &mut CompoundEnvironment::new(vec![]));
//!
//! assert!(matches!(term, Err(CompilationError::ParsingError(_, _))));
//! ```
//!
//! ```rust
//! use lambubu::{compile_term, CompoundEnvironment, Term, compile::CompilationError};
//! let term = compile_term("(\\x.x TEST)", &mut CompoundEnvironment::new(vec![]));
//!
//! assert!(matches!(term, Err(CompilationError::UnknownMacros { .. } )));
//! ```
use crate::{
    Term,
    env::{MutableTermEnvironment, TermEnvironment},
};
use pest::{
    Parser, Span,
    error::{Error, LineColLocation},
};
use pest_derive::Parser;
use std::rc::Rc;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CompilationError<'a> {
    #[error("Unknown macro {macro_name} at {span:?}")]
    UnknownMacros { macro_name: String, span: Span<'a> },
    #[error("Unexpected definition at {0:?}")]
    UnexpectedDefinition(Span<'a>),
    #[error("Parsing error")]
    ParsingError(&'a str, Box<Error<Rule>>),
}

impl<'a> From<CompilationError<'a>> for LineColLocation {
    fn from(val: CompilationError<'a>) -> LineColLocation {
        match val {
            CompilationError::UnknownMacros { span, .. }
            | CompilationError::UnexpectedDefinition(span) => LineColLocation::from(span),
            CompilationError::ParsingError(_, e) => e.line_col,
        }
    }
}

impl<'a> CompilationError<'a> {
    pub fn get_input(&self) -> &'a str {
        match self {
            Self::UnknownMacros { span, .. } | Self::UnexpectedDefinition(span) => span.get_input(),
            Self::ParsingError(e, _) => e,
        }
    }

    pub fn line_col(&self) -> LineColLocation {
        self.clone().into()
    }

    pub fn get_context(&self) -> String {
        match self {
            Self::ParsingError(_, e) => e.line().into(),
            Self::UnknownMacros { span, .. } | Self::UnexpectedDefinition(span) => {
                span.lines().collect::<Vec<_>>().join("\n")
            }
        }
    }
}

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn compile_pair<'a>(
    pair: Pair<'a>,
    env: &impl TermEnvironment,
) -> Result<Term, CompilationError<'a>> {
    match pair.as_rule() {
        Rule::Variable => Ok(Term::Var(Rc::from(pair.as_str()))),
        Rule::Abstraction => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str();
            let term = compile_pair(inner.next().unwrap(), env)?;

            Ok(Term::Abs(Rc::from(var_name), Rc::new(term)))
        }
        Rule::Application => {
            let mut inner = pair.into_inner().map(|x| compile_pair(x, env));
            let first = inner.next().unwrap()?;
            let second = inner.next().unwrap()?;
            let mut result = Term::Apply(Rc::new(first), Rc::new(second));

            for i in inner {
                result = Term::Apply(Rc::new(result), Rc::new(i?));
            }

            Ok(result)
        }
        Rule::MacrosName => {
            env.resolve_term(pair.as_str())
                .ok_or(CompilationError::UnknownMacros {
                    macro_name: pair.as_str().to_owned(),
                    span: pair.as_span(),
                })
        }
        Rule::Definition => Err(CompilationError::UnexpectedDefinition(pair.as_span())),
        _ => unreachable!(),
    }
}

/// Compiles a string slice into [crate::Term]
pub fn compile_term<'a>(
    input: &'a str,
    env: &impl TermEnvironment,
) -> Result<Term, CompilationError<'a>> {
    let parse_result = LambdaParser::parse(Rule::Term, input)
        .map_err(|x| CompilationError::ParsingError(input, Box::new(x)))?
        .next()
        .unwrap();

    compile_pair(parse_result, env)
}

/// Compiles a file into a vector of [crate::Term].
/// Also adds encountered macros to the environment.
pub fn compile_file<'a>(
    input: &'a str,
    env: &mut impl MutableTermEnvironment,
) -> Result<Vec<Term>, CompilationError<'a>> {
    let parse_result = LambdaParser::parse(Rule::File, input)
        .map_err(|x| CompilationError::ParsingError(input, Box::new(x)))?;
    let mut result = Vec::new();

    for pair in parse_result {
        match pair.as_rule() {
            Rule::Definition => {
                let mut inner = pair.into_inner();
                let macro_name = inner.next().unwrap().as_str();
                let term = compile_pair(inner.next().unwrap(), env)?;
                env.add_term(macro_name.to_owned(), term)
            }
            Rule::EOI => {}
            _ => result.push(compile_pair(pair, env)?),
        }
    }

    Ok(result)
}
