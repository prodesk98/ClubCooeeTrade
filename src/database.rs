use mongodb::{
    bson::{Document},
    Collection,
};
use futures::stream::StreamExt;

#[derive(Clone)]
pub struct Connection {
    collection: Collection<Document>,
}

impl Connection {
    pub fn new(collection: Collection<Document>) -> Self {
        Connection { collection }
    }

    pub async fn create(&self, doc: Document) -> mongodb::error::Result<()> {
        self.collection.insert_one(doc).await?;
        Ok(())
    }

    pub async fn read(&self, filter: Document) -> mongodb::error::Result<Vec<Document>> {
        let mut cursor = self.collection.find(filter).await?;
        let mut results = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => results.push(doc),
                Err(e) => eprintln!("Error reading document: {:?}", e),
            }
        }
        Ok(results)
    }
}