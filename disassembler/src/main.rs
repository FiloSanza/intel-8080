use std::process;

use disassembler;

fn main() {
    disassembler::run(&"/home/filippo/Progetti/intel-8080/rom/invaders/invaders")
        .unwrap_or_else(|err|{
            eprintln!("Error: {}", err);
            process::exit(1);            
        });
}
