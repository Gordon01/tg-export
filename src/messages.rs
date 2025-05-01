use std::time::{Duration, SystemTime};

use indexmap::IndexMap;
use serde::Deserialize;

use crate::{Reaction, Text, TextEntity};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RawMessage {
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
        #[serde(default)]
        reactions: Vec<Reaction>,
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

#[derive(Default)]
pub(crate) struct IndexedMessages {
    messages: IndexMap<u64, Message>,
}

pub(crate) struct Message {
    pub date: Option<SystemTime>,
    pub from: String,
    pub reply_to_message_id: Option<u64>,
    pub text: String,
    pub reactions: Vec<Reaction>,
    pub edited: Option<SystemTime>,
    pub text_entities: Vec<TextEntity>,
}

impl RawMessage {
    pub(crate) fn message(self) -> Option<(u64, Message)> {
        if let RawMessage::Message {
            id,
            reply_to_message_id,
            date_unixtime,
            from,
            text,
            reactions,
            edited_unixtime,
            text_entities,
            ..
        } = self
        {
            let date = date_unixtime
                .parse::<u64>()
                .map(|t| std::time::UNIX_EPOCH + Duration::from_secs(t))
                .ok();
            let edited = edited_unixtime
                .map(|d| {
                    d.parse::<u64>()
                        .map(|t| std::time::UNIX_EPOCH + Duration::from_secs(t))
                        .ok()
                })
                .flatten();
            let bm = Message {
                date,
                from: from.clone(),
                reply_to_message_id: reply_to_message_id,
                text: format!("{text}"),
                reactions,
                edited,
                text_entities,
            };
            Some((id, bm))
        } else {
            None
        }
    }
}

impl IndexedMessages {
    pub(crate) fn add_message(&mut self, id: u64, message: Message) {
        self.messages.insert(id, message);
    }
}
