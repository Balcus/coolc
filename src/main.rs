use clap::Parser;
use coolc::diagnostic::Diagnostic;
use coolc::lexer::LexerWrapper;
use coolc::parser;
use coolc::semantic_analysis::SemanticAnalyzer;
use coolc::string_table::StringTable;
use std::{fs, println};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to COOL source file
    path: String,

    /// Output destination for the produced binary
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<String>,

    /// Lexer debug mode
    #[arg(short = 'l')]
    lexer_debug: bool,

    /// Verbose lexer mode
    #[arg(short = 'v')]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.path).expect(&format!("Failed to read file: {}", cli.path));

    let mut s_table = StringTable::new();
    let mut errors = Vec::new();

    let tokens = Box::new(LexerWrapper::new(&input, &mut s_table, cli.path.clone()));

    let mut parser = parser::Parser::new(&cli.path, &mut errors);

    let program = match parser.parse(tokens) {
        Some(program) => program,
        None => {
            Diagnostic::new(cli.path.clone(), input.clone(), errors).emit_errors();
            return;
        }
    };

    if cli.verbose {
        println!("Generated Parse Tree:");
        println!("{:#?}", program);
        println!("{:#?}", s_table);
    } else {
        println!("{} passed parser checks", cli.path);
    }

    match SemanticAnalyzer::analyze(&program) {
        Ok(_ast) => {
            println!("{} passed semantic checks", cli.path);
        }
        Err(_semantic_errors) => {
            todo!()
            // let mut diagnostic = Diagnostic::new(cli.path.clone(), input.clone(), semantic_errors);
            // diagnostic.emit_errors();
        }
    }
}
