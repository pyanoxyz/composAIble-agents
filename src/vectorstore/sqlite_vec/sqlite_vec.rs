use std::{ collections::HashMap, error::Error, sync::Arc };

use async_trait::async_trait;
use serde_json::{ json, Value };
use sqlx::{ Pool, Row, Sqlite };

use crate::{
    embedding::embedder_trait::Embedder,
    schemas::document::Document,
    vectorstore::{ VecStoreOptions, VectorStore },
};

pub struct Store {
    pub pool: Pool<Sqlite>,
    pub(crate) table: String,
    pub(crate) embedder: Arc<dyn Embedder>,
}

impl Store {
    pub async fn initialize(&self) -> Result<(), Box<dyn Error>> {
        self.create_table_if_not_exists().await?;
        Ok(())
    }

    async fn create_table_if_not_exists(&self) -> Result<(), Box<dyn Error>> {
        let table = &self.table;

        sqlx
            ::query(
                &format!(
                    r#"
                CREATE TABLE IF NOT EXISTS {table}
                (
                  rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                  text TEXT,
                  metadata BLOB,
                  text_embedding BLOB
                )
                ;
                "#
                )
            )
            .execute(&self.pool).await?;

        let dimensions = self.embedder.dimensions();
        sqlx
            ::query(
                &format!(
                    r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS vec_{table} USING vec0(
                  text_embedding float[{dimensions}]
                );
                "#
                )
            )
            .execute(&self.pool).await?;

        // NOTE: python langchain seems to only use "embed_text" as the trigger name
        sqlx
            ::query(
                &format!(
                    r#"
                CREATE TRIGGER IF NOT EXISTS embed_text_{table}
                AFTER INSERT ON {table}
                BEGIN
                    INSERT INTO vec_{table}(rowid, text_embedding)
                    VALUES (new.rowid, new.text_embedding)
                    ;
                END;
                "#
                )
            )
            .execute(&self.pool).await?;

        Ok(())
    }

    fn get_filters(&self, opt: &VecStoreOptions) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        match &opt.filters {
            Some(Value::Object(map)) => {
                // Convert serde_json Map to HashMap<String, Value>
                let filters = map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Ok(filters)
            }
            None => Ok(HashMap::new()), // No filters provided
            _ => Err("Invalid filters format".into()), // Filters provided but not in the expected format
        }
    }

    fn build_metadata_query(&self, filter: &HashMap<String, Value>) -> String {
        if filter.is_empty() {
            return "TRUE".to_string();
        }

        filter
            .iter()
            .map(|(k, v)| {
                // Handle different JSON value types appropriately
                match v {
                    Value::String(s) => format!("json_extract(metadata, '$.{}') = '{}'", k, s),
                    Value::Number(n) => format!("json_extract(metadata, '$.{}') = {}", k, n),
                    Value::Bool(b) => format!("json_extract(metadata, '$.{}') = {}", k, b),
                    _ => format!("json_extract(metadata, '$.{}') = {}", k, v),
                }
            })
            .collect::<Vec<String>>()
            .join(" AND ")
    }
}

#[async_trait]
impl VectorStore for Store {
    async fn add_documents(
        &self,
        docs: &[Document],
        opt: &VecStoreOptions
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let texts: Vec<&str> = docs
            .iter()
            .map(|d| d.page_content.as_str())
            .collect();

        let embedder = opt.embedder.as_ref().unwrap_or(&self.embedder);

        let vectors = embedder.generate_embeddings_on_demand(&texts).await?;
        if vectors.len() != docs.len() {
            return Err(
                Box::new(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Number of vectors and documents do not match"
                    )
                )
            );
        }

        let table = &self.table;

        let mut tx = self.pool.begin().await?;

        let mut ids = Vec::with_capacity(docs.len());

        for (doc, vector) in docs.iter().zip(vectors.iter()) {
            let text_embedding = json!(&vector);
            let id = sqlx
                ::query(
                    &format!(
                        r#"
                    INSERT INTO {table}
                        (text, metadata, text_embedding)
                    VALUES
                        (?,?,?)"#
                    )
                )
                .bind(&doc.page_content)
                .bind(json!(&doc.metadata))
                .bind(text_embedding.to_string())
                .execute(&mut *tx).await?
                .last_insert_rowid();

            ids.push(id.to_string());
        }

        tx.commit().await?;
        println!("Documents added");
        Ok(ids)
    }

    async fn similarity_search(
        &self,
        query: &str,
        limit: usize,
        opt: &VecStoreOptions
    ) -> Result<Vec<Document>, Box<dyn Error>> {
        let table = &self.table;

        let embeddings = self.embedder.generate_embeddings_on_demand(&[query]).await?;

        let query_vector = match embeddings.get(0) {
            Some(query_embeddings) => json!(query_embeddings),
            None => {
                return Err("No embeddings returned".into());
            } // Handle the case where no embeddings are returned        }
        };

        let filter = self.get_filters(opt)?;

        let metadata_query = self.build_metadata_query(&filter);
        println!("Using metadata query = {}", metadata_query);
        // let rows = sqlx
        //     ::query(
        //         &format!(
        //             r#"SELECT
        //             text,
        //             metadata,
        //             distance
        //         FROM {table} e
        //         INNER JOIN vec_{table} v on v.rowid = e.rowid
        //         WHERE v.text_embedding match '{query_vector}' AND k = ? AND {metadata_query}
        //         ORDER BY distance
        //         LIMIT ?"#
        //         )
        //     )
        //     .bind(limit as i32)
        //     .fetch_all(&self.pool).await?;
        let rows = sqlx
            ::query(
                &format!(
                    r#"SELECT
                    e.text,
                    e.metadata,
                    v.distance
                FROM {table} e
                INNER JOIN vec_{table} v on v.rowid = e.rowid
                WHERE v.text_embedding match ? AND k = ? AND {metadata_query}
                ORDER BY v.distance ASC
                LIMIT ?"#
                )
            )
            .bind(query_vector.to_string()) // Ensure proper conversion
            .bind(limit as i32)
            .bind(limit as i32)
            .fetch_all(&self.pool).await?;

        let docs = rows
            .into_iter()
            .map(|row| {
                let page_content: String = row.try_get("text")?;
                let metadata_json: Value = row.try_get("metadata")?;
                let score: f64 = row.try_get("distance")?;

                let metadata = if let Value::Object(obj) = metadata_json {
                    obj.into_iter().collect()
                } else {
                    HashMap::new() // Or handle this case as needed
                };

                Ok(Document {
                    page_content,
                    metadata,
                    score,
                })
            })
            .collect::<Result<Vec<Document>, sqlx::Error>>()?;

        Ok(docs)
    }
}
