use clap::Parser;
use trex::app::App;
use trex::cli::Cli;
use trex::tmux::Tmux;

fn main() {
    let tmux = Tmux;
    let app = App::new(tmux);
    let cli = Cli::parse();

    match app.run(cli) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
