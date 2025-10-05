use mongodb::{Client, Database, bson::{doc, DateTime as BsonDateTime}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRecord {
    #[serde(rename = "_id")]
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_seen: BsonDateTime,
    pub last_interaction: BsonDateTime,
}

#[derive(Debug, Clone)]
pub struct DbClient {
    pub db: Database,
}

impl DbClient {
    pub async fn new(mongodb_uri: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(mongodb_uri).await?;
        let db = client.database("emoji_encoder_bot");

        Ok(Self { db })
    }

    pub async fn save_chat(
        &self,
        chat_id: i64,
        chat_type: String,
        title: Option<String>,
        username: Option<String>,
    ) -> Result<(), mongodb::error::Error> {
        let collection = self.db.collection::<ChatRecord>("chats");
        let now = BsonDateTime::now();

        let filter = doc! { "_id": chat_id };
        let update = doc! {
            "$set": {
                "chat_type": &chat_type,
                "title": title,
                "username": username,
                "last_interaction": now,
            },
            "$setOnInsert": {
                "first_seen": now,
            }
        };

        collection
            .update_one(filter, update)
            .upsert(true)
            .await?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<Stats, mongodb::error::Error> {
        let collection = self.db.collection::<ChatRecord>("chats");

        let total_chats = collection.count_documents(doc! {}).await?;

        let users = collection
            .count_documents(doc! { "chat_type": "private" })
            .await?;

        let groups = collection
            .count_documents(doc! { "chat_type": { "$in": ["group", "supergroup"] } })
            .await?;

        let channels = collection
            .count_documents(doc! { "chat_type": "channel" })
            .await?;

        Ok(Stats {
            total_chats,
            users,
            groups,
            channels,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_chats: u64,
    pub users: u64,
    pub groups: u64,
    pub channels: u64,
}
