use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Serialize;

use crate::Message;

#[derive(Clone, Debug, Default, Serialize)]
pub struct TextStats {
    pub count: u64,
    pub total_chars: u64,
    pub max_chars: u64,
}

impl TextStats {
    fn push(&mut self, chars: u64) {
        self.count += 1;
        self.total_chars += chars;
        self.max_chars = chars.max(self.max_chars);
    }
}

impl std::iter::Sum for TextStats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(TextStats::default(), |mut acc, item| {
            acc.count += item.count;
            acc.total_chars += item.total_chars;
            acc.max_chars = acc.max_chars.max(item.max_chars);
            acc
        })
    }
}

impl Display for TextStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count > 0 {
            write!(
                f,
                "- {} messages\n- Average: {} chars\n- Longest: {} chars",
                self.count,
                self.total_chars / self.count,
                self.max_chars
            )
        } else {
            write!(f, "- No messages")
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ChatStats {
    pub messages: u64,
    pub service_messages: u64,
    pub edited: u64,
    pub reactions: u64,
    pub participants: HashMap<String, TextStats>,
    pub text_entity_types: HashMap<String, u64>,
}

impl ChatStats {
    const HEAD_SIZE: usize = 5;

    pub fn analyze(messages: &[Message]) -> Self {
        let mut stats = ChatStats::default();
        stats.messages = messages.len() as u64;

        for message in messages {
            match message {
                Message::Message {
                    from,
                    text,
                    reactions,
                    edited,
                    edited_unixtime,
                    text_entities,
                    ..
                } => {
                    let len = text.to_string().chars().count() as u64;
                    stats
                        .participants
                        .entry(from.clone())
                        .or_default()
                        .push(len);

                    if let Some(r) = reactions {
                        stats.reactions += r.len() as u64;
                    }

                    if edited.is_some() || edited_unixtime.is_some() {
                        stats.edited += 1;
                    }

                    stats.count_entities(text_entities);
                }
                Message::Service { text_entities, .. } => {
                    stats.service_messages += 1;
                    stats.count_entities(text_entities);
                }
            }
        }

        stats
    }

    fn count_entities(&mut self, entities: &[crate::TextEntity]) {
        for entity in entities {
            *self
                .text_entity_types
                .entry(entity.entity_type.clone())
                .or_default() += 1;
        }
    }
}

impl fmt::Display for ChatStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📊 Chat Statistics Summary\n=========================")?;
        writeln!(f, "💬 Total messages: {}", self.messages)?;
        writeln!(f, "⚙️ Service messages: {}", self.service_messages)?;
        writeln!(f, "✏️ Edited messages: {}", self.edited)?;
        writeln!(f, "❤️ Total reactions: {}", self.reactions)?;

        let combined = self.participants.values().cloned().sum::<TextStats>();
        if combined.count > 0 {
            writeln!(f, "\n📏 Regular:\n{}", combined)?;
        }

        if !self.participants.is_empty() {
            let mut participants: Vec<_> = self.participants.iter().collect();
            participants.sort_unstable_by_key(|(_, stats)| std::cmp::Reverse(stats.count));

            writeln!(f, "\n👥 Top Participants ({}):", participants.len())?;
            for (i, (name, stats)) in participants.iter().take(Self::HEAD_SIZE).enumerate() {
                let percent = 100.0 * (stats.total_chars as f64 / combined.total_chars as f64);
                writeln!(f, "{}. {name} ({percent:.0}%)\n{stats}", i + 1)?;
            }
            if participants.len() > Self::HEAD_SIZE {
                writeln!(f, "... and {} more", participants.len() - Self::HEAD_SIZE)?;
            }
        }

        if !self.text_entity_types.is_empty() {
            writeln!(
                f,
                "\n🔤 Text Entity Types ({}):",
                self.text_entity_types.len()
            )?;
            let mut entities: Vec<_> = self.text_entity_types.iter().collect();
            entities.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));

            for (entity, &count) in entities {
                writeln!(f, "- {entity:15}: {count:>4}")?;
            }
        }

        Ok(())
    }
}
