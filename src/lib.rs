mod stats;

use std::{collections::HashMap, fmt::Display, io};

use serde::Deserialize;

pub use stats::{ChatStats, StatsSettings};

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
        id: u64,
        date: String,
        date_unixtime: String,
        from: String,
        from_id: String,
        reply_to_message_id: Option<u64>,
        text: Text,
        text_entities: Vec<TextEntity>,
        edited: Option<String>,
        edited_unixtime: Option<String>,
        reactions: Option<Vec<Reaction>>,
    },
    #[serde(rename = "service")]
    Service {
        id: u64,
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
        #[serde(default)]
        recent: Vec<RecentReaction>,
    },
    #[serde(rename = "custom_emoji")]
    CustomEmoji {
        count: i32,
        document_id: String,
        #[serde(default)]
        recent: Vec<RecentReaction>,
    },
}

#[derive(Debug, Deserialize)]
pub struct RecentReaction {
    pub from: String,
    pub from_id: String,
    pub date: String,
}

impl Chat {
    pub fn write_export<W: io::Write>(&self, writer: &mut W, max: Option<usize>) -> io::Result<()> {
        let mut messages = HashMap::new();
        let max = max.unwrap_or(self.messages.len());

        for msg in self.messages.iter().take(max) {
            if let Message::Message {
                id,
                date,
                from,
                text,
                edited,
                reactions,
                reply_to_message_id,
                ..
            } = msg
            {
                let msg_text = text.to_string().replace('\n', " ");
                messages.insert(id, (from.as_str(), msg_text.clone()));

                writeln!(writer, "[{}] @{}: {}", clean_date(date), from, msg_text)?;

                // Handle edit information
                if let Some(edited_date) = edited {
                    writeln!(writer, "  ↳ [edited] {}", clean_date(edited_date))?;
                }

                // Handle replies
                if let Some(reply_id) = reply_to_message_id {
                    if let Some((replied_from, replied_text)) = messages.get(&reply_id) {
                        writeln!(
                            writer,
                            "  ↳ [reply to msg#{}] @{}: {}",
                            reply_id, replied_from, replied_text
                        )?;
                    } else {
                        writeln!(writer, "  ↳ [reply to unknown msg#{}]", reply_id)?;
                    }
                }

                // Handle reactions
                if let Some(reactions) = reactions {
                    reactions.iter().try_for_each(|r| {
                        let (icon, users): (String, String) = match r {
                            Reaction::Emoji { emoji, recent, .. } => (
                                emoji.clone(),
                                recent.iter().map(|u| format!("@{}", u.from)).collect(),
                            ),
                            Reaction::CustomEmoji {
                                document_id,
                                recent,
                                ..
                            } => (
                                format!("custom_emoji:{}", document_id),
                                recent.iter().map(|u| format!("@{}", u.from)).collect(),
                            ),
                        };

                        writeln!(writer, "  ↳ [reaction: {} by {}]", icon, users)
                    })?;
                }
            }
        }

        Ok(())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Text::Plain(s) => write!(f, "{s}"),
            Text::Structured(elements) => elements
                .iter()
                .map(|e| match e {
                    TextElement::String(s) => write!(f, "{s}"),
                    TextElement::Entity(te) => write!(f, "{}", te.text),
                })
                .collect::<Result<_, _>>(),
        }
    }
}

fn clean_date(date: &str) -> String {
    date.replace('T', " ").replace('Z', "")
}
