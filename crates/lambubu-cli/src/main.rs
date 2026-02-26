use std::io;

use lambubu::{compile::compile_term, env::standard_environment};

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let mut term = compile_term(&buffer, &mut standard_environment()).unwrap();
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
    println!("{term}");
}
