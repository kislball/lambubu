use clap::Parser;
use lambubu::{BruijnLevelsTerm, CompoundEnvironment, Term, compile::compile_file};
use lambubu_church::ChurchEnvironment;
use std::{
    io::{self, Read},
    rc::Rc,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    #[arg(long, short, default_value = "false")]
    bruijn: bool,
}

fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

fn main() {
    let options = Options::parse();
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let terms = compile_file(&buffer, &mut standard_environment()).unwrap();
    println!("{}", options.bruijn);
    for (i, mut term) in terms.into_iter().enumerate() {
        const MAX_STEPS: usize = 10_000;
        let mut steps = 0;
        if options.bruijn {
            let mut term = BruijnLevelsTerm::from_open_term(term);
            while !term.clone().is_normal_form() {
                term = term.clone().reduce_step_applicative_order();
                steps += 1;
                if steps >= MAX_STEPS {
                    eprintln!("max steps exceeded");
                    return;
                }
            }
            let term = Into::<Term>::into(Rc::unwrap_or_clone(term));
            println!("{num}. {term}", num = i + 1);
        } else {
            while !term.is_normal_form() {
                term = term.reduce_step_applicative_order();
                steps += 1;
                if steps >= MAX_STEPS {
                    eprintln!("max steps exceeded");
                    return;
                }
            }
            println!("{num}. {term}", num = i + 1);
        }
    }
}
