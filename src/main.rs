use clap::Parser;
use coolc::lexer::{LexerExtras, Token};
use logos::Logos;
use std::fs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Paths for the input source files
    paths: Vec<String>,

    /// Output destination for the produced binary
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<String>,

    /// Lexer debug mode
    #[arg(short = 'l')]
    lexer_debug: bool,

    /// Verbose lexer mode
    #[arg(short = 'v')]
    lexer_verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    for path in &cli.paths {
        let s = fs::read_to_string(path).expect(&format!("Failed to read file: {path}"));
        if cli.lexer_verbose {
            let mut lexer = Token::lexer_with_extras(&s, LexerExtras::default());
            for res in lexer.by_ref() {
                match res {
                    Ok(token) => println!("{:#?}", token),
                    Err(e) => println!("Failed to read token {:?}", e),
                }
            }
            println!("\nString table:");
            for (string, id) in &lexer.extras.s_table.map {
                println!("  {id}: {string:?}");
            }
        }
    }
}
