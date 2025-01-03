use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{net::TcpListener, path::PathBuf};
use std::{fs, thread};
use std::sync::mpsc;
use colored::*;

#[derive(Parser)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long)]
    config: PathBuf,

    /// Run in daemon mode, outputting to a port
    #[arg(short, long)]
    daemon: Option<u16>,

    /// Number of lines to look back from the tail of the file
    #[arg(short, long, default_value_t = 100)]
    lookback: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    /// List of player names to look for in logs
    player_names: Vec<String>,

    /// Location of the client log file
    client_log_location: PathBuf,

    /// Filters to apply to log lines
    filters: Vec<String>,
}

enum Chat{
    Whisper,
    Party,
}


fn main() {
    let cli = Cli::parse();

    let config_content = fs::read_to_string(&cli.config)
        .expect("Unable to read configuration file");
    let config: Config = toml::from_str(&config_content)
        .expect("Invalid configuration file format");

    println!("{}", "Loaded configuration:".bold().blue());
    println!("{:?}", config);

    if let Some(port) = cli.daemon {
        run_as_daemon(port, config, cli.lookback);
    } else {
        // Normal execution logic
        println!("{}", "Running in standard mode.".bold().green());
        let logs = read_logs(&config.client_log_location, cli.lookback);
        for line in logs {
            println!("{}", line);
        }
    }
}

fn run_as_daemon(port: u16, config: Config, lookback: usize) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to port");
    println!("{}", format!("Daemon running on port {}", port).bold().yellow());

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        // Mock log processing loop
        let logs = read_logs(&config.client_log_location, lookback);
        for line in logs {
            if config.filters.iter().any(|filter| line.contains(filter)) {
                tx.send(line).unwrap();
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let messages: Vec<_> = rx.try_iter().collect();
                let response = messages.join("\n");
                use std::io::Write;
                writeln!(stream, "{}", response).expect("Failed to write to stream");
            }
            Err(e) => {
                eprintln!("{}", format!("Connection failed: {}", e).bold().red());
            }
        }
    }
}

fn read_logs(log_path: &PathBuf, lookback: usize) -> Vec<String> {
    let content = fs::read_to_string(log_path).expect("Unable to read log file");
    let lines: Vec<_> = content.lines().rev().take(lookback).collect();
    lines.into_iter().rev().map(String::from).collect()
}

// Example .toml file for the configuration
// player_names = ["Player1", "Player2"]
// client_log_location = "/path/to/client.log"
// filters = ["error", "warning"]
