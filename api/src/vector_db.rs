use anyhow::Result;
use openai::embeddings::Embedding;
use qdrant_client::prelude::QdrantClient;
use qdrant_client::qdrant::{
    points_selector::PointsSelectorOneOf, with_payload_selector::SelectorOptions, PointStruct,
    PointsIdsList, PointsSelector, SearchPoints, SearchResponse, WithPayloadSelector,
};
use serde_json::Value;

static COLLECTION_NAME: &str = "commands-v0";

pub struct VectorClient {
    client: QdrantClient,
}

impl VectorClient {
    pub async fn new(url: &str, token: &str) -> Result<Self> {
        let client = QdrantClient::from_url(url).with_api_key(token).build()?;
        Ok(Self { client })
    }

    pub async fn insert(&self, id: &str, embedding: Embedding, payload: Value) -> Result<()> {
        let vec: Vec<f32> = embedding.vec.iter().map(|&x| x as f32).collect();
        let payload = payload.try_into().unwrap();
        let points = vec![PointStruct::new(id.to_string(), vec, payload)];
        self.client
            .upsert_points(COLLECTION_NAME, None, points, None)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let point_selector_one_of = PointsSelectorOneOf::Points(PointsIdsList {
            ids: vec![id.to_string().try_into().unwrap()],
        });
        let points_selector = PointsSelector {
            points_selector_one_of: Some(point_selector_one_of),
        };
        self.client
            .delete_points(COLLECTION_NAME, None, &points_selector, None)
            .await?;

        Ok(())
    }

    pub async fn search(&self, query: Embedding, limit: u64) -> Result<SearchResponse> {
        let vec: Vec<f32> = query.vec.iter().map(|&x| x as f32).collect();
        let payload_selector = WithPayloadSelector {
            selector_options: Some(SelectorOptions::Enable(true)),
        };

        let search_points = SearchPoints {
            collection_name: COLLECTION_NAME.to_string(),
            vector: vec,
            limit,
            with_payload: Some(payload_selector),
            ..Default::default()
        };

        let search_result = self.client.search_points(&search_points).await.unwrap();

        Ok(search_result)
    }
}
