use simple_risc::emulator::Emulator;
use simple_risc::parser::parse_and_assemble;
use std::{env::args, process::exit};

fn main() {
    let code: String;
    if let Some(p) = args().nth(1) {
        let path = std::path::Path::new(&p);
        code = std::fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Cannot read file: {}", err);
            exit(1);
        });
    } else {
        eprintln!(
            "Usage: {} <filepath>",
            args().next().unwrap_or_else(|| String::from("simpleRISC"))
        );
        exit(1);
    }

    let instructions = parse_and_assemble(&code).unwrap_or_else(|err| {
        eprintln!("[ERROR] {}", err);
        exit(1);
    });
    let mut emul = Emulator::new(&instructions);
    emul.exec().unwrap_or_else(|err| {
        eprintln!("[ERROR] {}", err);
        exit(1);
    });
    emul.debug();
}
