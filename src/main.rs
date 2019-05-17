mod cli;
mod config;
mod plexpy;
mod printer;

fn main() {
    if let Err(err) = cli::execute() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
