use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input source code files
    files: Vec<String>,

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

    for _file in &cli.files {
        todo!();
    }
}
