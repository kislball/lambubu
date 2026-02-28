use std::io::{self, Read};

use lambubu::{CompoundEnvironment, compile::compile_file};
use lambubu_church::ChurchEnvironment;

fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let terms = compile_file(&buffer, &mut standard_environment()).unwrap();
    for (i, mut term) in terms.into_iter().enumerate() {
        const MAX_STEPS: usize = 10_000;
        let mut steps = 0;
        while !term.is_normal_form() {
            term = term.reduce_step_normal_order();
            steps += 1;
            if steps >= MAX_STEPS {
                eprintln!("max steps exceeded");
                return;
            }
        }
        println!("{num}. {term}", num = i + 1);
    }
}
