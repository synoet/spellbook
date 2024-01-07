use anyhow::Result;
use git2::{Oid, Repository};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing;

#[derive(Deserialize, Serialize, Debug)]
pub struct PushWebhookPayload {
    pub after: String,
    pub before: String,
    pub head_commit: CommitPayload,
    pub repository: RepositoryPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitPayload {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
    pub author: AuthorPayload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RepositoryPayload {
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthorPayload {
    pub username: String,
    pub email: String,
}

fn get_file_content(repo: &git2::Repository, tree: &git2::Tree, file: &str) -> String {
    let path = Path::new(file);
    let blob = tree
        .get_path(path)
        .unwrap()
        .to_object(&repo)
        .unwrap()
        .into_blob()
        .unwrap();
    let content = String::from_utf8(blob.content().to_vec()).unwrap();
    content
}

pub struct ProcessedPushPayload {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<(String, String)>,
}

pub fn process_payload(payload: PushWebhookPayload) -> Result<ProcessedPushPayload> {
    tracing::info!(
        "Processing webhook push payload for commit {}, from user {}({})",
        payload.after,
        payload.head_commit.author.username,
        payload.head_commit.author.email
    );
    if std::path::Path::new("/tmp/spellbook/repo").exists() {
        std::fs::remove_dir_all("/tmp/spellbook/repo").unwrap();
    }
    let repo_path = "/tmp/spellbook/repo";
    let repo = Repository::clone(&payload.repository.url, repo_path)?;

    let get_commit = |hash: &str| repo.find_commit(Oid::from_str(hash).unwrap()).unwrap();

    let curr_commit = get_commit(&payload.after);
    let prev_commit = get_commit(&payload.before);

    let curr_tree = curr_commit.tree()?;
    let prev_tree = prev_commit.tree()?;

    let only_json = |path: &str| path.ends_with(".json");

    let removed_files = payload
        .head_commit
        .removed
        .iter()
        .filter(|path| only_json(path));
    let added_files = payload
        .head_commit
        .added
        .iter()
        .filter(|path| only_json(path));
    let modified_files = payload
        .head_commit
        .modified
        .iter()
        .filter(|path| only_json(path));

    let modified_files_diff: Vec<(String, String)> = modified_files
        .map(|file| {
            return (
                get_file_content(&repo, &curr_tree, file),
                get_file_content(&repo, &prev_tree, file),
            );
        })
        .collect::<Vec<(String, String)>>();

    let removed_files_content: Vec<String> = removed_files
        .map(|file| return get_file_content(&repo, &prev_tree, file))
        .collect::<Vec<String>>();

    let added_files_content: Vec<String> = added_files
        .map(|file| {
            let content = get_file_content(&repo, &curr_tree, file);
            return content;
        })
        .collect::<Vec<String>>();

    Ok(ProcessedPushPayload {
        added: added_files_content,
        removed: removed_files_content,
        modified: modified_files_diff,
    })
}
