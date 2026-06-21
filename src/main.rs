use clap::Parser;
use coolc::lexer::LexerWrapper;
use coolc::parser;
use coolc::s_table::StringTable;
use std::fs;

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
    lexer_verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.path).expect(&format!("Failed to read file: {}", cli.path));
    let mut s_table = StringTable::new();
    let _lexer = LexerWrapper::new(&input, &mut s_table);
    let _parser = parser::ProgramParser::new();
}
