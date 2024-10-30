mod ast;
mod hook;
mod inst;
mod r#match;
mod visit;

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use colored::Colorize;
use inst::Instrumenter;

#[derive(Parser)]
#[command(name = "Pass Inst")]
struct PassInst {
    target: String,

    #[arg(short, long, default_value = "./instrumented")]
    output: String,
}

fn main() {
    let pass_inst = PassInst::parse();
    let path = Path::new(&pass_inst.target);
    if !path.exists() {
        eprintln!(
            "{} {} does not exist!",
            "Error".red().bold(),
            path.display()
        );
        return;
    }

    let mut work_list: Vec<PathBuf> = vec![];

    if path.is_file() {
        if path.extension().unwrap() == "cpp" {
            work_list.push(path.to_path_buf());
        } else {
            println!("{} only instrument xxx.cpp!", "Warning".yellow().bold());
        }
    }

    if path.is_dir() {
        for entry in path.read_dir().expect("Failed to read the directory!") {
            if let Ok(e) = entry {
                let file_path = e.path();
                if file_path.is_file() {
                    if file_path.extension().unwrap() == "cpp" {
                        work_list.push(file_path.to_path_buf());
                    } else {
                        println!("{} only instrument xxx.cpp!", "Warning".yellow().bold());
                    }
                }
            }
        }
    }

    if work_list.is_empty() {
        println!("{} No file to instrument, exit.", "Finished".green().bold());
    }

    let mut instrumenter = Instrumenter::new();
    let instrument = |path: &PathBuf| {
        let absolute_path = path.canonicalize().unwrap();
        let mut code = fs::read_to_string(absolute_path).unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();

        instrumenter.instrument(filename, &mut code);

        let output_filename = pass_inst.output.to_owned() + "/" + filename;
        fs::write(output_filename, code).unwrap();
    };

    work_list.iter().for_each(instrument);
}
