use crate::Term;
use crate::env::MutableTermEnvironment;
use pest::{Parser, Span};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CompilationError<'a> {
    #[error("Unknown macro {macro_name} at {span:?}")]
    UnknownMacros { macro_name: String, span: Span<'a> },
}

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn compile_pair<'a>(
    pair: Pair<'a>,
    env: &mut impl MutableTermEnvironment,
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
            let mut inner = pair.into_inner();
            let term_a = compile_pair(inner.next().unwrap(), env)?;
            let term_b = compile_pair(inner.next().unwrap(), env)?;

            Ok(Term::Apply(Box::new(term_a), Box::new(term_b)))
        }
        Rule::MacrosName => {
            env.resovle_term(pair.as_str())
                .ok_or(CompilationError::UnknownMacros {
                    macro_name: pair.as_str().to_owned(),
                    span: pair.as_span(),
                })
        }
        _ => unreachable!(),
    }
}

pub fn compile_term<'a>(
    input: &'a str,
    env: &mut impl MutableTermEnvironment,
) -> Result<Term, CompilationError<'a>> {
    let parse_result = LambdaParser::parse(Rule::File, input)
        .unwrap()
        .next()
        .unwrap();

    compile_pair(parse_result, env)
}
