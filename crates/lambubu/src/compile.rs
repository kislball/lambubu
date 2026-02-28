use crate::{
    Term,
    env::{MutableTermEnvironment, TermEnvironment},
};
use pest::{Parser, Span};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CompilationError<'a> {
    #[error("Unknown macro {macro_name} at {span:?}")]
    UnknownMacros { macro_name: String, span: Span<'a> },
    #[error("Unexpected definition at {0:?}")]
    UnexpectedDefinition(Span<'a>),
}

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn compile_pair<'a>(
    pair: Pair<'a>,
    env: &impl TermEnvironment,
) -> Result<Term, CompilationError<'a>> {
    match pair.as_rule() {
        Rule::Variable => Ok(Term::Var(pair.as_str().to_owned())),
        Rule::Abstraction => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str().to_owned();
            let term = compile_pair(inner.next().unwrap(), env)?;

            Ok(Term::Abs(var_name, Box::new(term)))
        }
        Rule::Application => {
            let mut inner = pair.into_inner().map(|x| compile_pair(x, env));
            let first = inner.next().unwrap()?;
            let second = inner.next().unwrap()?;
            let mut result = Term::Apply(Box::new(first), Box::new(second));

            for i in inner {
                result = Term::Apply(Box::new(result), Box::new(i?));
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

pub fn compile_term<'a>(
    input: &'a str,
    env: &impl TermEnvironment,
) -> Result<Term, CompilationError<'a>> {
    let parse_result = LambdaParser::parse(Rule::Term, input)
        .unwrap()
        .next()
        .unwrap();

    compile_pair(parse_result, env)
}

pub fn compile_file<'a>(
    input: &'a str,
    env: &mut impl MutableTermEnvironment,
) -> Result<Vec<Term>, CompilationError<'a>> {
    let parse_result = LambdaParser::parse(Rule::File, input).unwrap();
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
