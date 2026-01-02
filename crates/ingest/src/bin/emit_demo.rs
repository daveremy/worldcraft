use std::io::{self, Write};
use std::path::Path;

fn arg_value(args: &[String], flag: &str, default: &str) -> String {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(String::from)
        .unwrap_or_else(|| default.to_string())
}

fn load_events(path: &Path) -> Vec<serde_json::Value> {
    let data = std::fs::read_to_string(path).expect("failed to read demo events");
    serde_json::from_str(&data).expect("failed to parse demo events")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = arg_value(&args, "--input", "data/demo-events.json");
    let events = load_events(Path::new(&input_path));

    let mut stdout = io::BufWriter::new(io::stdout());
    for event in events {
        let line = serde_json::to_string(&event).expect("failed to serialize event");
        writeln!(stdout, "{line}").expect("failed to write event");
    }
}
