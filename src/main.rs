use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader},
    net::TcpListener,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};
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

    /// Output format: json, toml, jsonl, yaml
    #[arg(short, long, default_value = "json")]
    output_as: String,
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

#[derive(Serialize, Deserialize, Debug)]
struct LogLine<'a> {
    timestamp: Cow<'a, str>,
    id: Cow<'a, str>,
    code: Cow<'a, str>,
    log_level: Cow<'a, str>,
    client: Cow<'a, str>,
    message: Cow<'a, str>,
}

fn main() {
    let cli = Cli::parse();

    let config_content = fs::read_to_string(&cli.config)
        .expect("Unable to read configuration file");
    let config: Config = toml::from_str(&config_content)
        .expect("Invalid configuration file format");

    // println!("{}", "Loaded configuration:".bold().blue());
    // println!("{:?}", config);

    let log_cache = Arc::new(Mutex::new(VecDeque::new()));

    if let Some(port) = cli.daemon {
        run_as_daemon(port, config, cli.lookback, log_cache);
    } else {
        // println!("{}", "Running in standard mode.".bold().green());
        let rx = read_logs(&config.client_log_location, cli.lookback, Arc::clone(&log_cache));
        for line in rx {
            if let Some(parsed_line) = parse_log_line(&line) {
                output_as_requested(&parsed_line, &cli.output_as);
            }
        }
    }
}

fn run_as_daemon(
    port: u16,
    config: Config,
    lookback: usize,
    log_cache: Arc<Mutex<VecDeque<String>>>,
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to port");
    // println!("{}", format!("Daemon running on port {}", port).bold().yellow());

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let log_cache = Arc::clone(&log_cache);
        let rx = read_logs(&config.client_log_location, lookback, log_cache);
        for line in rx {
            if config.filters.iter().any(|filter| line.contains(filter)) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let messages: Vec<_> = rx.try_iter().collect();
                let response = serde_json::to_string(&messages).expect("Failed to serialize log lines");
                use std::io::Write;
                writeln!(stream, "{}", response).expect("Failed to write to stream");
            }
            Err(e) => {
                eprintln!("{}", format!("Connection failed: {}", e).bold().red());
            }
        }
    }
}

fn read_logs<'a>(
    log_path: &PathBuf,
    lookback: usize,
    log_cache: Arc<Mutex<VecDeque<String>>>,
) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn({
        let log_cache = Arc::clone(&log_cache);
        let log_path = log_path.clone();
        move || {
            let file = fs::File::open(&log_path).expect("Unable to open log file");
            let reader = BufReader::new(file);

            let lines: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();
            let recent_lines: Vec<_> = lines.iter().rev().take(lookback).cloned().collect();

            let mut cache = log_cache.lock().unwrap();
            for line in &recent_lines {
                cache.push_back(line.clone());
                if tx.send(line.clone()).is_err() {
                    break;
                }
            }

            cache.truncate(lookback);
        }
    });

    rx
}

fn parse_log_line<'a>(line: &'a str) -> Option<LogLine<'a>> {
    let parts: Vec<&str> = line.splitn(6, ' ').collect();
    if parts.len() < 6 {
        return None;
    }
    Some(LogLine {
        timestamp: Cow::Owned(parts[0].to_string() + " " + parts[1]),
        id: Cow::Borrowed(parts[2]),
        code: Cow::Borrowed(parts[3]),
        log_level: Cow::Borrowed(parts[4]),
        client: Cow::Borrowed(parts[5]),
        message: Cow::Borrowed(parts.get(6).unwrap_or(&"")),
    })
}

fn output_as_requested(log_line: &LogLine, format: &str) {
    match format {
        "json" => println!("{}", serde_json::to_string(log_line).expect("Failed to serialize to JSON")),
        "toml" => println!("{}", toml::to_string(log_line).expect("Failed to serialize to TOML")),
        "jsonl" => println!("{}", serde_json::to_string(log_line).expect("Failed to serialize to JSONL")),
        "yaml" => println!("{}", serde_yaml::to_string(log_line).expect("Failed to serialize to YAML")),
        _ => eprintln!("{}", "Unsupported format. Defaulting to JSON.".bold().red()),
    }
}
