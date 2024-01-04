use crate::command::SubCommand;
use anyhow::{Error, Result};
use openai::embeddings::Embedding;
use shuttle_secrets::SecretStore;

pub fn initialize_openai(secrets: &SecretStore) -> Result<()> {
    let openai_token = secrets.get("OPENAI_TOKEN").unwrap();
    openai::set_key(openai_token);
    Ok(())
}

pub async fn embed_command(command: SubCommand) -> Result<Embedding, Error> {
    let embedding =
        Embedding::create("text-embedding-ada-002", &command.to_string(), "spellbook").await?;

    Ok(embedding)
}

pub async fn embed_query(query: &str) -> Result<Embedding, Error> {
    let embedding = Embedding::create("text-embedding-ada-002", query, "spellbook").await?;

    Ok(embedding)
}
