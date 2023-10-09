use simple_risc::emulator::Emulator;
use simple_risc::parser::parse_and_assemble;
use std::{env::args, io::Write, process::exit};

fn main() {
    if !matches!(args().count(), 2 | 3) {
        eprintln!(
            "Usage: {} <filepath>",
            args().next().unwrap_or_else(|| String::from("simpleRISC"))
        );
        exit(1);
    }

    let code: String;
    if let Some(p) = args().nth(1) {
        let path = std::path::Path::new(&p);
        code = std::fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Cannot read file: {}", err);
            exit(1);
        });
    } else {
        exit(1);
    }

    let instructions = parse_and_assemble(&code).unwrap_or_else(|err| {
        eprintln!("[ERROR] {}", err);
        exit(1);
    });

    // Write assembled binary to file if outfile name given
    if let Some(outpath) = args().nth(2) {
        let mut outfile = std::fs::File::create(&outpath).unwrap_or_else(|err| {
            eprintln!("[ERROR] {}. Cannot open outfile '{}'", err, outpath);
            exit(1);
        });

        for ins in &instructions {
            outfile.write(&ins.to_le_bytes()).unwrap_or_else(|err| {
                eprintln!("[ERROR] {}. Cannot write to outfile {}", err, outpath);
                exit(1);
            });
        }
    }

    let mut emul = Emulator::new(&instructions);
    emul.exec().unwrap_or_else(|err| {
        eprintln!("[ERROR] {}", err);
        exit(1);
    });
    emul.debug();
}
