use crate::Term;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse_pair(pair: Pair) -> Result<Term, ()> {
    match pair.as_rule() {
        Rule::Variable => Ok(Term::Var(pair.as_str().to_owned())),
        Rule::Abstraction => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str().to_owned();
            let term = parse_pair(inner.next().unwrap())?;

            Ok(Term::Abs(var_name, Box::new(term)))
        }
        Rule::Application => {
            let mut inner = pair.into_inner();
            let term_a = parse_pair(inner.next().unwrap())?;
            let term_b = parse_pair(inner.next().unwrap())?;

            Ok(Term::Apply(Box::new(term_a), Box::new(term_b)))
        }
        _ => {
            panic!(
                "Unhandled rule: {:?}, span: {:?}",
                pair.as_rule(),
                pair.as_str()
            );
        }
    }
}

pub fn parse_term(input: &str) -> Result<Term, ()> {
    let parse_result = LambdaParser::parse(Rule::File, input)
        .unwrap()
        .next()
        .unwrap();

    parse_pair(parse_result)
}
