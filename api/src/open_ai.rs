use crate::command::SubCommand;
use anyhow::{Error, Result};
use openai::embeddings::Embedding;

pub fn initialize_openai(token: String) -> Result<()> {
    openai::set_key(token);
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
