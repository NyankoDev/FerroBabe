use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::ExitCode,
};

use ferro_babe::{Diagnostic, Disassembler, FerroFormatter, Formatter};

fn main() -> ExitCode {
    run().unwrap_or_else(|message| {
        eprintln!("ferro-disassemble: {message}");
        ExitCode::FAILURE
    })
}

fn run() -> Result<ExitCode, String> {
    let path = class_path()?;
    let file =
        File::open(&path).map_err(|error| format!("could not open {}: {error}", path.display()))?;
    let disassembly = Disassembler::default()
        .parse_reader(file)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;

    let version = disassembly.version();
    let Some(class) = disassembly.class() else {
        println!(
            "partial class file v{}.{}",
            version.major(),
            version.minor()
        );
        for diagnostic in disassembly.diagnostics() {
            print_diagnostic(diagnostic);
        }
        return Ok(ExitCode::from(2));
    };

    let output = FerroFormatter
        .format(class)
        .map_err(|error| format!("could not format {}: {error}", path.display()))?;
    print!("{output}");

    Ok(ExitCode::SUCCESS)
}

fn class_path() -> Result<PathBuf, String> {
    let mut arguments = env::args_os();
    let executable = arguments.next().unwrap_or_default();
    let Some(path) = arguments.next() else {
        return Err(usage(&executable));
    };

    if arguments.next().is_some() {
        return Err(usage(&executable));
    }

    Ok(PathBuf::from(path))
}

fn usage(executable: &std::ffi::OsStr) -> String {
    format!("usage: {} <class-file>", Path::new(executable).display())
}

fn print_diagnostic(diagnostic: &Diagnostic) {
    let offset = diagnostic
        .offset()
        .map(|offset| format!(" at code offset 0x{offset:04x}"))
        .unwrap_or_default();

    eprintln!(
        "diagnostic: {:?} in {:?}{offset}: {}",
        diagnostic.severity(),
        diagnostic.stage(),
        diagnostic.message()
    );
}
