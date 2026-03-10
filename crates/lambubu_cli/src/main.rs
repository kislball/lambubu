use clap::Parser;
use lambubu::{BruijnLevelsTerm, CompoundEnvironment, Term, compile::compile_file};
use lambubu_church::ChurchEnvironment;
use std::io::{self, Read};

fn reduce_bruijn_applicative(term: BruijnLevelsTerm) -> (BruijnLevelsTerm, bool) {
    term.reduce_step_applicative_order()
}

fn reduce_bruijn_normal(term: BruijnLevelsTerm) -> (BruijnLevelsTerm, bool) {
    term.reduce_step_normal_order()
}

fn reduce_bruijn_cbn(term: BruijnLevelsTerm) -> (BruijnLevelsTerm, bool) {
    term.reduce_step_call_by_name()
}

fn reduce_bruijn_cbv(term: BruijnLevelsTerm) -> (BruijnLevelsTerm, bool) {
    term.reduce_step_call_by_value()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Options {
    #[arg(long, short, default_value = "false")]
    bruijn: bool,

    #[arg(long, short, value_enum, default_value = "applicative")]
    strategy: Strategy,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Strategy {
    Applicative,
    Normal,
    CallByName,
    CallByValue,
}

pub fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

fn reduce_applicative(term: Term) -> (Term, bool) {
    term.reduce_step_applicative_order()
}

fn reduce_normal(term: Term) -> (Term, bool) {
    term.reduce_step_normal_order()
}

fn reduce_cbn(term: Term) -> (Term, bool) {
    term.reduce_step_call_by_name()
}

fn reduce_cbv(term: Term) -> (Term, bool) {
    term.reduce_step_call_by_value()
}

fn main() {
    let options = Options::parse();
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let terms = compile_file(&buffer, &mut standard_environment()).unwrap();

    let reducer: fn(Term) -> (Term, bool) = match options.strategy {
        Strategy::Applicative => reduce_applicative,
        Strategy::Normal => reduce_normal,
        Strategy::CallByName => reduce_cbn,
        Strategy::CallByValue => reduce_cbv,
    };

    for (i, mut term) in terms.into_iter().enumerate() {
        const MAX_STEPS: usize = 10_000;
        let mut steps = 0;
        if options.bruijn {
            let bruijn_reducer: fn(BruijnLevelsTerm) -> (BruijnLevelsTerm, bool) =
                match options.strategy {
                    Strategy::Applicative => reduce_bruijn_applicative,
                    Strategy::Normal => reduce_bruijn_normal,
                    Strategy::CallByName => reduce_bruijn_cbn,
                    Strategy::CallByValue => reduce_bruijn_cbv,
                };
            let mut term = BruijnLevelsTerm::from_open_term(term);
            loop {
                let (next, changed) = bruijn_reducer(term);
                term = next;
                if !changed {
                    break;
                }
                steps += 1;
                if steps >= MAX_STEPS {
                    eprintln!("max steps exceeded");
                    return;
                }
            }
            eprintln!("Steps: {}", steps);
            let term = Into::<Term>::into(term);
            println!("{num}. {term}", num = i + 1);
        } else {
            while !term.is_normal_form() {
                let (next, changed) = reducer(term);
                term = next;
                if !changed {
                    break;
                }
                steps += 1;
                if steps >= MAX_STEPS {
                    eprintln!("max steps exceeded");
                    return;
                }
            }
            eprintln!("Steps: {}", steps);
            println!("{num}. {term}", num = i + 1);
        }
    }
}
