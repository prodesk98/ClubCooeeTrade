use mongodb::{
    bson::{Document},
    Collection,
};
use futures::stream::StreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;

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
        // limit 20
        let options = FindOptions::builder().limit(20).sort(doc! {"_id": -1}).build();
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