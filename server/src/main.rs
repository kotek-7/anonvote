use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    arg: String,
}

fn main() {
    let cli = Cli::parse();
    println!("{}", cli.arg);
}
