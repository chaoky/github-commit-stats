mod cli;
mod github;
mod histogram;

use crate::github::LanguageStats;
use clap::Parser;
use plotters::style::*;
use std::fs;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = cli::Args::parse();

    let language_stats: Vec<(String, LanguageStats)> = match (args.cache_path, args.personal_token)
    {
        (Some(path), None) => serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap(),
        (None, Some(personal_token)) => github::Client::new(&personal_token).run().await,
        _ => {
            panic!("please provide either a personal token or a path to the cached file")
        }
    };

    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.changes),
        YELLOW,
        "stats changes",
    );
    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.additions),
        GREEN,
        "stats additions",
    );
    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.deletions),
        RED,
        "stats deletions",
    );
}
