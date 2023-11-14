mod config;
mod language;
mod message;
mod phase;

use config::Config;
use std::{collections::HashMap, path::PathBuf, process::ExitCode};

use crate::{
    message::Message,
    phase::{
        ast_builder::AstBuilder,
        interpreter::Interpreter,
        lexer::Lexer,
        parser::Parser,
        phase::{Phase, PhaseResult},
        type_checker::TypeChecker,
    },
};

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Runs the current SFL project
    Run,
}

fn main() -> ExitCode {
    use clap::Parser;
    let cli = Cli::parse();

    // TODO: Get config from clap
    let config = Config::default();

    match &cli.command {
        Commands::Run => match run(config) {
            Ok(_) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        },
    }
}

/// Look for a 'main.sfl' file in the current folder and run it
fn run(config: Config) -> Result<(), ()> {
    let mut paths = std::fs::read_dir("./").unwrap();

    let entry_point = match paths.find(|p| p.as_ref().is_ok_and(|p| p.file_name() == "main.sfl")) {
        Some(Ok(p)) => p.path().clone(),
        _ => {
            eprintln!("No 'main.sfl' found in current project!");
            return Err(());
        }
    };

    let contents = std::fs::read_to_string(&entry_point).expect("Unable to read 'main.sfl'");
    let mut sources = HashMap::new();
    sources.insert(entry_point, contents);

    let lexer_result = Lexer::new().run(&config, &sources);
    let lexer_result = complete_phase(&sources, &config, lexer_result)?;

    let parser_result = Parser::new().run(&config, &lexer_result);
    let parser_result = complete_phase(&sources, &config, parser_result)?;

    println!(
        "{}",
        parser_result
            .get(&PathBuf::from("./main.sfl"))
            .unwrap()
            .pretty_print()
    );

    // let ast_result = AstBuilder::new().run(&config, &parser_result);
    // let ast_result = complete_phase(&sources, &config, ast_result)?;

    // println!(
    //     "--- AST ------------\n{}",
    //     ast_result
    //         .get(&PathBuf::from("./main.sfl"))
    //         .unwrap()
    //         .pretty_print()
    // );

    // let typechecker_result = TypeChecker::new().run(&config, &ast_result);
    // let typechecker_result = complete_phase(&sources, &config, typechecker_result)?;

    // println!(
    //     "\n--- TYPE -----------\n{:?}",
    //     typechecker_result
    //         .get(&PathBuf::from("./main.sfl"))
    //         .unwrap()
    // );

    // let result = Interpreter::new().run(&config, &ast_result);
    // let result = complete_phase(&sources, &config, result);

    // println!("\n--- RESULT -----------\n{}", result.unwrap());

    Ok(())
}

fn complete_phase<R>(
    sources: &HashMap<PathBuf, String>,
    config: &Config,
    phase_result: PhaseResult<R>,
) -> Result<R, ()> {
    match phase_result {
        PhaseResult::Ok(result) => Ok(result),
        PhaseResult::SoftErr(result, errors) => {
            println!("{}", Message::format_errors(sources, &errors));
            if !config.resilient {
                Err(())
            } else {
                Ok(result)
            }
        }
        PhaseResult::Err(errors) => {
            println!("{}", Message::format_errors(sources, &errors));
            Err(())
        }
    }
}
