#![feature(result_option_inspect)]

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

    let language_stats = match (args.cache_path, args.personal_token) {
        (Some(path), _) => {
            serde_json::from_str::<Vec<(String, LanguageStats)>>(&fs::read_to_string(path).unwrap())
                .unwrap()
        }
        (_, Some(personal_token)) => github::Client::new(&personal_token).run().await,
        (None, None) => {
            panic!("please provide either a personal token or a path to the cached file")
        }
    };

    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.changes),
        YELLOW,
        "stats_changes.png",
    );
    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.additions),
        GREEN,
        "stats_additions.png",
    );
    histogram::draw(
        &language_stats,
        |(lang, stats)| (lang.to_string(), stats.deletions),
        RED,
        "stats_deletions.png",
    );
}
