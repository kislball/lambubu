use std::io;

use lambubu::parse::parse_term;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let mut term = parse_term(&buffer).unwrap();
    println!("{term}");
    while term != term.clone().reduce_step_normal_order() {
        term = term.reduce_step_normal_order();
    }
    println!("{term}");
}
