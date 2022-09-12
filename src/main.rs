use simple_risc::parser;
use std::{env::args, process::exit};

fn main() {
    let code: String;

    match args().nth(1) {
        Some(p) => {
            let fpath = std::path::Path::new(&p);
            if let Ok(txt) = std::fs::read_to_string(fpath) {
                code = txt + "\n";
                let mut asm = parser::Parser::from(&code);
                match asm.parse() {
                    Ok(bins) => {
                        for bin in bins {
                            println!("{:032b}", bin);
                        }
                    }
                    Err(err) => asm.print_err(err),
                }
            } else {
                eprintln!("Cannot read file!");
                exit(1);
            }
        }
        None => {
            eprintln!("Usage: {} <filepath>", args().nth(0).unwrap());
            exit(1);
        }
    }
}