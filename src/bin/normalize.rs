use std::process::ExitCode;

use itn_indic::Normalizer;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(input), None) = (arguments.next(), arguments.next()) else {
        eprintln!("Usage: normalize \"Malayalam sentence\"");
        return ExitCode::from(2);
    };
    let Ok(input) = input.into_string() else {
        eprintln!("Input must be valid UTF-8");
        return ExitCode::from(2);
    };

    match Normalizer::new("ml").and_then(|normalizer| normalizer.normalize(&input)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: {error}", error.code());
            ExitCode::FAILURE
        }
    }
}
