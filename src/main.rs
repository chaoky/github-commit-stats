mod cli;
mod github;
mod histogram;

use crate::github::LanguageStats;
use clap::Parser;
use hyperpolyglot::Language;
use itertools::Itertools;
use plotters::style::*;
use std::fs;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = cli::Args::parse();

    let mut language_stats: Vec<(Option<Language>, LanguageStats)> =
        match (args.cache_path, args.personal_token) {
            (Some(path), None) => serde_json::from_str::<Vec<(String, LanguageStats)>>(
                &fs::read_to_string(path).unwrap(),
            )
            .unwrap()
            .into_iter()
            .map(|(lang, stats)| (Language::try_from(lang.as_str()).ok(), stats))
            .collect(),

            (None, Some(personal_token)) => github::Client::new(&personal_token).run().await,

            _ => {
                panic!("please provide either a personal token or a path to the cached file")
            }
        };

    language_stats.retain(|(lang, _)| {
        lang.map(|x| {
            if args.hidden.contains(&x.name.into()) {
                return false;
            };
            let categories = args.categories.clone();
            if categories.is_empty() {
                return true;
            }
            categories.contains(&x.language_type.into())
        })
        .unwrap_or(args.group_threshold != 0)
    });

    histogram::draw(
        language_stats
            .iter()
            .map(|(lang, stats)| {
                (
                    lang.map(|x| x.name).unwrap_or("Others").to_string(),
                    stats.changes,
                )
            })
            .collect_vec(),
        YELLOW,
        "stats changes",
        args.group_threshold,
    );
    histogram::draw(
        language_stats
            .iter()
            .map(|(lang, stats)| {
                (
                    lang.map(|x| x.name).unwrap_or("Others").to_string(),
                    stats.additions,
                )
            })
            .collect_vec(),
        GREEN,
        "stats additions",
        args.group_threshold,
    );
    histogram::draw(
        language_stats
            .iter()
            .map(|(lang, stats)| {
                (
                    lang.map(|x| x.name).unwrap_or("Others").to_string(),
                    stats.deletions,
                )
            })
            .collect_vec(),
        RED,
        "stats deletions",
        args.group_threshold,
    );
}
