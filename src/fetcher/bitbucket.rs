use std::cmp::Ordering;

use anyhow::Result;
use itertools::Itertools;
use reqwest::{Client, header::HeaderMap};
use rustc_hash::FxHashMap;
use rustyline::completion::Pair;
use serde::Deserialize;
use versions::Versioning;

use crate::{
    cfg::AccessTokens,
    fetcher::{Fetcher, PackageInfo, Revisions, Version, json, success},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchFromBitbucket {
    owner: String,
    repo: String,
}

#[derive(Deserialize)]
struct Repo {
    description: String,
}

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
    #[serde(default)]
    body: String,
}

#[derive(Deserialize)]
struct Reference {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Deserialize)]
struct Commit {
    sha: String,
    commit: CommitInfo,
}

#[derive(Deserialize)]
struct CommitInfo {
    committer: Committer,
    message: String,
}

#[derive(Deserialize)]
struct Committer {
    date: String,
}

impl Fetcher for FetchFromBitbucket {
    async fn create_client(&self, mut tokens: AccessTokens) -> Result<Client> {
        let mut headers = HeaderMap::new();
        tokens.insert_header(&mut headers, "bitbucket.org").await;
        Client::builder()
            .user_agent("Mozilla/5.0")
            .default_headers(headers)
            .build()
            .map_err(Into::into)
    }

    async fn get_package_info(&self, cl: &Client) -> PackageInfo {
        let root = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}",
            self.owner, self.repo,
        );

        let (description, tags, commits) = tokio::join!(
            async {
                json(cl, &root)
                    .await
                    .map_or_else(String::new, |repo: Repo| repo.description)
            },
            json::<Vec<_>>(cl, format!("{root}/git/matching-refs/tags/")),
            json::<Vec<_>>(cl, format!("{root}/commits?per_page=12")),
        );

        let mut completions = vec![];
        let mut versions = FxHashMap::default();

        if let Some(tags) = tags {
            let mut tags = tags
                .into_iter()
                .filter_map(|Reference { reference }| {
                    reference.strip_prefix("refs/tags/").map(ToOwned::to_owned)
                })
                .sorted_unstable_by(|x, y| match (&Versioning::new(x), &Versioning::new(y)) {
                    (Some(x), Some(y)) => y.cmp(x),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => y.cmp(x),
                })
                .take(12);

            for tag in tags {
                completions.push(Pair {
                    display: format!("{tag} (tag)"),
                    replacement: tag.clone(),
                });
                versions.insert(tag, Version::Tag);
            }
        }

        if let Some(commits) = commits {
            let mut commits = commits.into_iter();

            if let Some(Commit { sha, commit }) = commits.next() {
                let date = &commit.committer.date[0..10];
                let msg = commit.message.lines().next().unwrap_or_default();

                completions.push(Pair {
                    display: format!("{sha} ({date} - HEAD) {msg}"),
                    replacement: sha.clone(),
                });
                versions.insert(
                    sha,
                    Version::Head {
                        date: date.into(),
                        msg: msg.into(),
                    },
                );
            }

            for Commit { sha, commit } in commits {
                let date = &commit.committer.date[0..10];
                let msg = commit.message.lines().next().unwrap_or_default();
                completions.push(Pair {
                    display: format!("{sha} ({date}) {msg}"),
                    replacement: sha.clone(),
                });
                versions.insert(
                    sha,
                    Version::Commit {
                        date: date.into(),
                        msg: msg.into(),
                    },
                );
            }
        };

        let homepage = format!("https://bitbucket.org/{}/{}", self.owner, self.repo);

        PackageInfo {
            pname: self.repo.clone(),
            description,
            file_url_prefix: Some(format!("{homepage}/blob/${{finalAttrs.src.rev}}/")),
            homepage,
            license: Vec::new(),
            python_dependencies: Default::default(),
            releases_page: None,
            revisions: Revisions {
                latest,
                completions,
                versions,
            },
        }
    }

    async fn get_version(&self, cl: &Client, rev: &str) -> Option<Version> {
        let Commit { sha, commit } = json(
            cl,
            format!(
                "https://api.bitbucket.org/repos/{}/{}/commits/{rev}",
                self.owner, self.repo,
            ),
        )
        .await?;

        Some(if sha.starts_with(rev) {
            Version::Commit {
                date: commit.committer.date[0..10].into(),
                msg: "".into(),
            }
        } else {
            Version::Tag
        })
    }

    async fn has_submodules(&self, cl: &Client, rev: &str) -> bool {
        success(
            cl,
            format!(
                "https://api.bitbucket.org/repos/{}/{}/contents/.gitmodules?ref={rev}",
                self.owner, self.repo,
            ),
        )
        .await
    }
}
