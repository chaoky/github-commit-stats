use futures::prelude::*;
use hyperpolyglot::Language;
use octocrab::{
    models::{repos::RepoCommit, Repository},
    Octocrab,
};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, AddAssign},
    path::Path,
};
use tempfile::TempDir;
use tokio::fs::write;
use tracing::warn;

pub struct Client {
    client: Octocrab,
    temp_dir: TempDir,
}

impl Client {
    pub fn new(personal_token: &str) -> Self {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(personal_token.to_string())
            .build()
            .unwrap();

        let temp_dir = tempfile::tempdir_in("/dev/shm").unwrap();

        Self { client, temp_dir }
    }

    pub async fn run(&self) -> Vec<(Option<Language>, LanguageStats)> {
        self.repos()
            .flat_map_unordered(None, |repo| Box::pin(self.list_commits(repo)))
            .flat_map_unordered(None, |commit| Box::pin(self.commit_details(commit)))
            .fold(Vec::new(), |mut acc, (lang, stats)| async move {
                match acc.iter().position(|(acc_lang, _)| acc_lang == &lang) {
                    Some(pos) => acc[pos].1 += stats,
                    None => acc.push((lang, stats)),
                };
                acc
            })
            .await
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
    ) -> impl Stream<Item = (Option<Language>, LanguageStats)> + '_ {
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
                    .and_then(|x| Language::try_from(x.language()).ok()),
                LanguageStats {
                    additions: file.get("additions").unwrap().as_u64().unwrap() as usize,
                    deletions: file.get("deletions").unwrap().as_u64().unwrap() as usize,
                    changes: file.get("changes").unwrap().as_u64().unwrap() as usize,
                },
            )
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageStats {
    pub additions: usize,
    pub deletions: usize,
    pub changes: usize,
}

impl AddAssign for LanguageStats {
    fn add_assign(&mut self, rhs: Self) {
        self.additions += rhs.additions;
        self.changes += rhs.changes;
        self.deletions += rhs.deletions;
    }
}

impl Add for &LanguageStats {
    type Output = LanguageStats;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            additions: self.additions + rhs.additions,
            deletions: self.changes + rhs.changes,
            changes: self.deletions + rhs.deletions,
        }
    }
}
