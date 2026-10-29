use clap::Parser;

#[derive(Parser)]
struct Cli {
    search: String,
}

fn main() {
    let args = Cli::parse();
    let history_file_path = std::path::PathBuf::from("~/.zsh_history");
    let history = std::fs::read_to_string(history_file_path).expect("Could not read history");
    let history_lines = history.lines();

    for (i, line) in history_lines.enumerate() {
        if line.contains(&args.search) {
            println!("{} {}", i, line)
        }
    }

    println!("search: {:?}", args.search);
}
