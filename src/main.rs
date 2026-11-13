use clap::Parser;
use coolc::codegen::codegen;
use coolc::diagnostic::Diagnostic;
use coolc::inheritance_tree::InheritanceTree;
use coolc::lexer::LexerWrapper;
use coolc::parser;
use coolc::semantic_analysis::SemanticAnalyzer;
use coolc::semantic_analysis::method_table::MethodTable;
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
    let parse_res = parser.parse(tokens);

    if parse_res.is_none() {
        Diagnostic::new(cli.path.clone(), input.clone(), errors).emit_errors(&s_table);
        std::process::exit(1);
    }

    let parse_tree = parse_res.unwrap();

    if cli.verbose {
        println!("Generated Parse Tree:");
        println!("{:#?}", parse_tree);
        println!("{:#?}", s_table);
    } else {
        println!("{} passed parser checks", cli.path);
    }

    let inheritance_tree = InheritanceTree::build(&parse_tree).unwrap_or_else(|errors| {
        Diagnostic::from_semantic_errors(cli.path.clone(), input.clone(), errors)
            .emit_errors(&s_table);

        std::process::exit(1);
    });

    let method_table = MethodTable::build(&parse_tree).unwrap_or_else(|errors| {
        Diagnostic::from_semantic_errors(cli.path.clone(), input.clone(), errors)
            .emit_errors(&s_table);

        std::process::exit(1);
    });

    let ast_root = SemanticAnalyzer::analyze(parse_tree, &inheritance_tree, &method_table)
        .unwrap_or_else(|errors| {
            Diagnostic::from_semantic_errors(cli.path.clone(), input.clone(), errors)
                .emit_errors(&s_table);

            std::process::exit(1);
        });

    println!("{} passed semantic checks", cli.path);

    let res = codegen(ast_root, &method_table, &s_table);
    println!("{res}");
}
