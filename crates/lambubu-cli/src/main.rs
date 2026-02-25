use std::io;

use lambubu::{compile::compile_term, env::RegistryEnvironment};

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let mut term = compile_term(&buffer, &mut RegistryEnvironment::new()).unwrap();
    println!("{term}");
    while term != term.clone().reduce_step_normal_order() {
        term = term.reduce_step_normal_order();
    }
    println!("{term}");
}
