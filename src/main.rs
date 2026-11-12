mod cli;

fn main() {
    let args = std::env::args_os()
        .skip(1)
        .map(|argument| argument.into_string())
        .collect::<Result<Vec<_>, _>>();

    match args {
        Ok(args) => std::process::exit(cli::run(args)),
        Err(_) => {
            eprintln!("ERROR: Command-line arguments must use UTF-8.");
            std::process::exit(2);
        }
    }
}
