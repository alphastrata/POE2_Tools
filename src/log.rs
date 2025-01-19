//$ src/log.rs
use clap::Parser;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc};
use std::{borrow::Cow, collections::{HashMap, VecDeque}, fs, io::{BufRead, BufReader}, net::TcpListener, path::PathBuf, sync::{mpsc, Arc, Mutex, OnceLock}, thread};
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
    #[arg(short, long, default_value_t = 1_000_000)]
    lookback: usize,

    /// Output format: json, toml, jsonl, yaml and the default: stdout
    #[arg(short, long, default_value = "stdout")]
    output_as: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    player_names: Vec<String>,
    client_log_location: PathBuf,
    always_include: Vec<String>,
    always_exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogLine<'a> {
    timestamp: i64,
    id: Cow<'a, str>,
    code: Cow<'a, str>,
    log_level: Cow<'a, str>,
    client_num: Option<u32>,
    message: Cow<'a, str>,
    nested: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
enum Session {
    All,
    Latest,
    Recent(usize),
}

#[derive(Debug)]
enum PoeLogLvl {
    Info,
    Critical,
    Debug,
    Warn,
}

static LOG_CACHE: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
static DEATHS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
static SESSION_START: OnceLock<Mutex<NaiveDateTime>> = OnceLock::new();

fn main() {
    let cli = Cli::parse();

    let config_content = fs::read_to_string(&cli.config)
        .expect("Unable to read configuration file");
    let config= toml::from_str(&config_content)
    .expect("Invalid configuration file format");


    LOG_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    DEATHS.get_or_init(|| Mutex::new(HashMap::new()));
    SESSION_START.get_or_init(|| Mutex::new(Utc::now().naive_utc()));

    if let Some(port) = cli.daemon {
        run_as_daemon(port, config, cli.lookback);
    } else {
        let config_a: Arc<Config> = Arc::new(config);

        let rx = read_logs(config_a, cli.lookback);
        for line in rx {
            if let Some(parsed_line) = parse_log_line(&line) {
                update_deaths(&parsed_line);
                output_as_requested(&parsed_line, &cli.output_as);
            }
        }
    }
}

fn run_as_daemon(port: u16, config: Config, lookback: usize) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to port");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let log_lines = read_logs(config.clone().into(), lookback);
        for line in log_lines {
            if config.always_include.iter().any(|include| line.contains(include)) && tx.send(line).is_err() {
                break;
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
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn read_logs<'a>(config: Arc<Config>, lookback: usize) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let file = fs::File::open(&config.client_log_location).expect("Unable to open log file");
        let reader = BufReader::new(file);

        let lines: Vec<String> = reader.lines().filter_map(|line| {
            let line = line.ok()?;
            if config.always_exclude.iter().any(|exclusion| line.contains(exclusion)) {
                None
            } else {
                Some(line)
            }
        }).collect();

        let recent_lines: Vec<_> = lines.iter().rev().take(lookback).cloned().collect();
        for line in recent_lines {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    rx
}

fn parse_log_line(line: &str) -> Option<LogLine<'_>> {
    let parts: Vec<&str> = line.splitn(6, ' ').collect();
    if parts.len() < 6 {
        return None;
    }

    let timestamp_str = format!("{} {}", parts[0], parts[1]);
    let timestamp = NaiveDateTime::parse_from_str(&timestamp_str, "%Y/%m/%d %H:%M:%S")
        .ok()?
        .and_utc().timestamp();

    let log_level = parts[4].trim_matches(|c| c == '[' || c == ']');
    let nested = if parts[5].contains('[') {
        Some(Cow::Borrowed(parts[5].split('[').nth(1).unwrap_or("").trim_matches(|c| c == ']')))
    } else {
        None
    };

    Some(LogLine {
        timestamp,
        id: Cow::Borrowed(parts[2]),
        code: Cow::Borrowed(parts[3]),
        log_level: Cow::Borrowed(log_level),
        client_num: parts[5].split(' ').nth(1).and_then(|n| n.parse::<u32>().ok()),
        message: Cow::Borrowed(parts.get(6).unwrap_or(&"")),
        nested,
    })
}

fn update_deaths(log_line: &LogLine) {
    if log_line.message.contains("has been slain") {
        let player = log_line.message.split_whitespace().next().unwrap_or("").to_string();
        let mut deaths = DEATHS.get().unwrap().lock().unwrap();
        *deaths.entry(player).or_insert(0) += 1;
    }
}

impl std::fmt::Display for LogLine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = NaiveDateTime::from_timestamp_opt(self.timestamp, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "<invalid timestamp>".to_string());

        writeln!(
            f,
            "${} \n${} \n${} \n${} \n${}\n\nPLAYER_LVL = {}\nPLAYING_AS = {}\nCURRENT_SESSION = {}\nStarted at = {}",
            "most recent log line -4".bright_white(),
            "most recent log line -3".bright_white(),
            "most recent log line -2".bright_white(),
            "most recent log line -1".bright_white(),
            "most recent log line".bright_white(),
            "<level_placeholder>".bright_magenta(), // Replace with actual player level extraction logic
            "<player_name_placeholder>".bright_cyan(), // Replace with actual player name extraction logic
            timestamp.bright_yellow(), // Derived timestamp for session time
            "<start_time_placeholder>".bright_white(), // Replace with app start time logic
        )
    }
}

fn output_as_requested(log_line: &LogLine, format: &str) {
    match format {
        "json" => println!("{}", serde_json::to_string(log_line).expect("Failed to serialize to JSON")),
        "toml" => println!("{}", toml::to_string(log_line).expect("Failed to serialize to TOML")),
        "jsonl" => println!("{}", serde_json::to_string(log_line).expect("Failed to serialize to JSONL")),
        "yaml" => println!("{}", serde_yaml::to_string(log_line).expect("Failed to serialize to YAML")),
        _ => println!("{}", log_line), 
    }
}
