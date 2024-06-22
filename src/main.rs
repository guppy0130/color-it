use chrono::{DateTime, FixedOffset};
use clap::{ArgAction::Count, Parser};
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
    /// json key to read for for level/severity
    #[arg(short, long, default_value = "level")]
    level: String,
    /// json key to read for log message
    #[arg(short, long, default_value = "message")]
    message: String,
    /// json key to read for log timestamp
    #[arg(short, long, default_value = "timestamp")]
    timestamp: String,
    /// how to parse timestamp (defaults to ISO8601)
    #[arg(short, long, default_value = "%+")]
    strptime: String,
    /// how much color to use (0..2, 0 is just level)
    #[arg(short, long, action=Count)]
    color_amount: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    env_logger::builder()
        .format(move |buf, record| {
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
                dt.to_rfc3339().if_supports_color(Stdout, |text| {
                    if config.color_amount >= 2 {
                        text.style(style)
                    } else {
                        text.style(Style::new())
                    }
                }),
                record
                    .level()
                    .if_supports_color(Stdout, |text| text.style(style)),
                record.args().if_supports_color(Stdout, |text| {
                    if config.color_amount >= 1 {
                        text.style(style)
                    } else {
                        text.style(Style::new())
                    }
                })
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
        let ts_str: String = match &v[config.timestamp.clone()] {
            serde_json::Value::Number(v) => v.as_f64().unwrap().to_string(),
            serde_json::Value::String(v) => v.into(),
            _ => panic!(),
        };
        // this is suspicious; if it doesn't deserialize to utf8 then uh...
        let m = v[config.message.clone()].as_str().unwrap_or_default();

        let mut kvs: BTreeMap<&str, &str> = BTreeMap::new();
        kvs.insert("timestamp", ts_str.as_str());
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
