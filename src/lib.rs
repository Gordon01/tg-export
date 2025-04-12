mod stats;

use serde::Deserialize;

pub use stats::ChatStats;

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub name: String,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub id: i64,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "message")]
    Message {
        id: i64,
        date: String,
        date_unixtime: String,
        from: String,
        from_id: String,
        text: Text,
        text_entities: Vec<TextEntity>,
        edited: Option<String>,
        edited_unixtime: Option<String>,
        reactions: Option<Vec<Reaction>>,
    },
    #[serde(rename = "service")]
    Service {
        id: i64,
        date: String,
        date_unixtime: String,
        actor: String,
        actor_id: String,
        action: String,
        duration_seconds: Option<u32>,
        discard_reason: Option<String>,
        text: Text,
        text_entities: Vec<TextEntity>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Text {
    Plain(String),
    Structured(Vec<TextElement>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextElement {
    String(String),
    Entity(TextEntity),
}

#[derive(Debug, Deserialize)]
pub struct TextEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Reaction {
    #[serde(rename = "emoji")]
    Emoji {
        count: i32,
        emoji: String,
        recent: Vec<RecentReaction>,
    },
    #[serde(rename = "custom_emoji")]
    CustomEmoji {
        count: i32,
        document_id: String,
        recent: Vec<RecentReaction>,
    },
}

#[derive(Debug, Deserialize)]
pub struct RecentReaction {
    pub from: String,
    pub from_id: String,
    pub date: String,
}
