use simple_risc::emulator::Emulator;
use simple_risc::parser::Parser;
use std::{env::args, process::exit};

fn main() {
    let code: String;
    if let Some(p) = args().nth(1) {
        let path = std::path::Path::new(&p);
        if let Ok(txt) = std::fs::read_to_string(path) {
            code = txt;
        } else {
            eprintln!("Cannot read file!");
            exit(1);
        }
    } else {
        eprintln!(
            "Usage: {} <filepath>",
            args().next().unwrap_or_else(|| String::from("simpleRISC"))
        );
        exit(1);
    }

    let instructions: Vec<u32>;
    let mut asm = Parser::from(&code);
    match asm.parse() {
        Ok(bins) => {
            println!("Parsed successfully.");
            instructions = bins;
        }
        Err(err) => {
            asm.print_err(err);
            exit(1)
        }
    }
    let mut emul = Emulator::from(&instructions);
    if let Err(e) = emul.exec() {
        eprintln!("Error: {:?}", e);
        exit(1);
    }
    emul.debug();
}
