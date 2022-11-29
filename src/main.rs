#![feature(result_option_inspect)]

use futures::prelude::*;
use octocrab::{
    models::{repos::RepoCommit, Repository},
    Octocrab,
};
use plotters::prelude::SVGBackend;
use plotters::prelude::*;
use std::{collections::HashMap, ops::AddAssign, path::Path};
use tempfile::TempDir;
use tokio::fs::write;
use tracing::warn;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    App::new().run().await;
}

struct App {
    client: Octocrab,
    temp_dir: TempDir,
}

impl App {
    fn new() -> Self {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token("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".to_string())
            .build()
            .unwrap();

        let temp_dir = tempfile::tempdir_in("/dev/shm").unwrap();

        Self { client, temp_dir }
    }

    async fn run(&self) {
        // let language_count: HashMap<Language, LanguageStats> = self
        //     .repos()
        //     .flat_map_unordered(None, |repo| Box::pin(self.list_commits(repo)))
        //     .flat_map_unordered(None, |commit| Box::pin(self.commit_details(commit)))
        //     .fold(HashMap::new(), |mut acc, (lang, stats)| async move {
        //         acc.entry(lang)
        //             .and_modify(|x| *x += stats.clone())
        //             .or_insert(stats);
        //         acc
        //     })
        //     .await;

        let mut languages = vec![
            ("Rust", 35603),
            ("Lisp", 1737),
            ("Others", 2229),
            ("Scala", 2353),
            ("TypeScript", 9838),
        ];
        languages.sort_by(|(_, a), (_, b)| a.cmp(b));

        let root = SVGBackend::new("stats.svg", (640, 480)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .margin(50)
            .caption("Language Diff Stats", ("sans-serif", 50.0))
            .build_cartesian_2d(
                0usize..languages.iter().fold(0, |acc, (_, count)| acc + count),
                (0..languages.len() - 1).into_segmented(),
            )
            .unwrap();

        chart
            .configure_mesh()
            .disable_y_mesh()
            .y_label_formatter(&|x: &SegmentValue<usize>| match x {
                SegmentValue::CenterOf(x) => languages[*x].0.to_owned(),
                _ => unreachable!(),
            })
            .draw()
            .unwrap();

        chart
            .draw_series(
                Histogram::horizontal(&chart)
                    .style(RED.mix(0.5).filled())
                    .data(
                        languages
                            .iter()
                            .enumerate()
                            .map(|(index, (_, count))| (index, *count)),
                    ),
            )
            .unwrap();

        root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    }

    fn repos(&self) -> impl Stream<Item = Repository> + '_ {
        stream::once(async move {
            self.client
                .current()
                .list_repos_for_authenticated_user()
                .affiliation("owner, collaborator, organization_member")
                .per_page(100)
                .send()
                .await
                .unwrap()
        })
        .flat_map(stream::iter)
    }

    fn list_commits(&self, repo: Repository) -> impl Stream<Item = RepoCommit> + '_ {
        stream::once(async move {
            self.client
                .repos(&repo.owner.clone().unwrap().login, &repo.name)
                .list_commits()
                .author("chaoky")
                .per_page(100)
                .send()
                .await
                .unwrap()
        })
        .flat_map(stream::iter)
    }

    fn commit_details(
        &self,
        commit: RepoCommit,
    ) -> impl Stream<Item = (Language, LanguageStats)> + '_ {
        stream::once(async move {
            self.client
                ._get(&commit.url, None::<&()>)
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()
        })
        .flat_map(|details| stream::iter(details.get("files").unwrap().as_array().unwrap().clone()))
        .map(|file| {
            let path = self.temp_dir.path().join(
                Path::new(file.get("filename").unwrap().as_str().unwrap())
                    .file_name()
                    .unwrap(),
            );
            (file, path)
        })
        .then(|(file, path)| async move {
            write(
                path.clone(),
                file.get("raw_url").unwrap().as_str().unwrap_or_else(|| {
                    warn!("no raw_url for {:?}", path);
                    ""
                }),
            )
            .await
            .unwrap();
            (file, path)
        })
        .map(|(file, path)| {
            (
                hyperpolyglot::detect(&path)
                    .unwrap()
                    .map(|x| x.language())
                    .unwrap_or("unknown")
                    .to_string(),
                LanguageStats {
                    additions: file.get("additions").unwrap().as_u64().unwrap(),
                    deletions: file.get("deletions").unwrap().as_u64().unwrap(),
                    changes: file.get("changes").unwrap().as_u64().unwrap(),
                },
            )
        })
    }
}

type Language = String;
#[derive(Debug, Clone)]
struct LanguageStats {
    additions: u64,
    deletions: u64,
    changes: u64,
}

impl AddAssign for LanguageStats {
    fn add_assign(&mut self, rhs: Self) {
        self.additions += rhs.additions;
        self.changes += rhs.changes;
        self.deletions += rhs.deletions;
    }
}
