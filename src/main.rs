use chrono::{DateTime, FixedOffset};
use clap::Parser;
use log::{logger, Level, RecordBuilder};
use owo_colors::{OwoColorize, Stream::Stdout, Style};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, BufRead, Write},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Config {
    #[arg(short, long, default_value = "level")]
    level: String,
    #[arg(short, long, default_value = "message")]
    message: String,
    #[arg(short, long, default_value = "timestamp")]
    timestamp: String,
    #[arg(short, long, default_value = "%+")]
    strptime: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    env_logger::builder()
        .format(|buf, record| {
            let kv = record.key_values();
            let ts_str: &str = &kv.get("timestamp".into()).unwrap().to_string();
            // defaulting to ISO8601 because we're sane and we don't want to just guess
            let fmt: &str = &kv.get("strptime".into()).unwrap_or("%+".into()).to_string();
            let dt: DateTime<FixedOffset> = DateTime::parse_from_str(ts_str, fmt)
                .unwrap_or_else(|_| panic!("Failed to parse {} with {}", ts_str, fmt));

            // colorize the message
            let style = match record.level() {
                Level::Trace => Style::new().cyan(),
                Level::Debug => Style::new().blue(),
                Level::Info => Style::new().green(),
                Level::Warn => Style::new().yellow(),
                Level::Error => Style::new().red(),
            };

            writeln!(
                buf,
                "{} {: <6} - {}",
                // wait, how much do I color?
                dt.to_rfc3339(),
                record
                    .level()
                    .if_supports_color(Stdout, |text| text.style(style)),
                record
                    .args()
                    .if_supports_color(Stdout, |text| text.style(style))
            )
        })
        .init();

    let mut buffer = String::new();
    let mut stdin = io::stdin().lock();
    let l = logger();

    while stdin.read_line(&mut buffer)? != 0 {
        // skip empty lines (nothing to output, but maybe there's more later?)
        if buffer.trim().is_empty() {
            continue;
        }

        let mut v: serde_json::Value = serde_json::from_str(&buffer)?;
        // ok, we just hope that you're outputting to a way upstream has decided to deserialize.
        let level: Level = serde_json::from_value(v[config.level.clone()].take())?;
        // wow, I hope we can deserialize that timestamp without having to support other formats!
        let ts_str: &str = v[config.timestamp.clone()]
            .as_str()
            .unwrap_or("1970-01-01 00:00:00");
        // this is suspicious; if it doesn't deserialize to utf8 then uh...
        let m = v[config.message.clone()].as_str().unwrap_or_default();

        let mut kvs: BTreeMap<&str, &str> = BTreeMap::new();
        kvs.insert("timestamp", ts_str);
        kvs.insert("strptime", &config.strptime);
        let mut record = RecordBuilder::new();
        l.log(
            &record
                .level(level)
                .args(format_args!("{}", m))
                .key_values(&kvs)
                .build(),
        );
        buffer.clear();
    }
    Ok(())
}
