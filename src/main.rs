use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use csv::WriterBuilder;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = File::open(args.input)?;
    let reader = BufReader::new(file);
    let data: Vec<Value> = serde_json::from_reader(reader)?;

    let mut headers = BTreeSet::new();
    let mut records = Vec::new();

    for item in data {
        if let Value::Object(map) = item {
            let mut record = BTreeMap::new();

            for (key, val) in map {
                if key != "_id" {
                    headers.insert(key.clone());
                }
                let formatted = match val {
                    Value::Array(arr) if arr.iter().all(|v| v.is_number()) => {
                        arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
                    }
                    Value::Array(arr) if arr.iter().all(|v| v.is_string()) => {
                        arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
                    }
                    _ => val.to_string().trim_matches('"').to_string(),
                };
                if key != "_id" {
                    record.insert(key, formatted);
                }
            }

            records.push(record);
        }
    }

    let mut writer = WriterBuilder::new().from_path(args.output)?;
    writer.write_record(&headers)?;

    for record in records {
        let row: Vec<String> = headers.iter().map(|h| record.get(h).cloned().unwrap_or_default()).collect();
        writer.write_record(row)?;
    }

    writer.flush()?;
    Ok(())
}
