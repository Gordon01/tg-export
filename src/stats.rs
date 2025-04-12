use std::{collections::HashMap, fmt};

use serde::Serialize;

use crate::{Message, Text, TextElement};

#[derive(Debug, Default, Serialize)]
pub struct ChatStats {
    pub total: u64,
    pub messages: u64,
    pub service_messages: u64,
    pub edited: u64,
    pub reactions: u64,
    pub participants: HashMap<String, u64>,
    pub message_lengths: (u64, u64), // (total_chars, max)
    pub text_entity_types: HashMap<String, u64>,
}

impl ChatStats {
    pub fn analyze(messages: &[Message]) -> Self {
        let mut stats = ChatStats::default();
        let mut length_counts = Vec::new();

        for message in messages {
            stats.total += 1;
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
                    stats.messages += 1;

                    // Count participant messages
                    *stats.participants.entry(from.clone()).or_insert(0) += 1;

                    // Count message length
                    let length = match text {
                        Text::Plain(s) => s.chars().count(),
                        Text::Structured(te) => te
                            .iter()
                            .map(|e| match e {
                                TextElement::String(s) => s.chars().count(),
                                TextElement::Entity(te) => te.text.chars().count(),
                            })
                            .sum(),
                    } as u64;
                    length_counts.push(length);
                    stats.message_lengths.0 += length;

                    // Count reactions
                    if let Some(reactions) = reactions {
                        stats.reactions += reactions.len() as u64;
                    }

                    // Count edited messages
                    if edited.is_some() || edited_unixtime.is_some() {
                        stats.edited += 1;
                    }

                    // Count entity types
                    for entity in text_entities {
                        *stats
                            .text_entity_types
                            .entry(entity.entity_type.clone())
                            .or_insert(0) += 1;
                    }
                }
                Message::Service {
                    actor,
                    text_entities,
                    ..
                } => {
                    stats.service_messages += 1;
                    *stats.participants.entry(actor.clone()).or_insert(0) += 1;

                    // Count entity types for service messages
                    for entity in text_entities {
                        *stats
                            .text_entity_types
                            .entry(entity.entity_type.clone())
                            .or_insert(0) += 1;
                    }
                }
            }
        }

        // Calculate min/max lengths
        if !length_counts.is_empty() {
            stats.message_lengths.1 = *length_counts.iter().max().unwrap_or(&0);
        }

        stats
    }
}

impl fmt::Display for ChatStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📊 Chat Statistics Summary")?;
        writeln!(f, "=========================")?;
        writeln!(f, "📝 Total messages: {}", self.total)?;
        writeln!(f, "💬 Regular messages: {}", self.messages)?;
        writeln!(f, "⚙️ Service messages: {}", self.service_messages)?;
        writeln!(f, "✏️ Edited messages: {}", self.edited)?;
        writeln!(f, "❤️ Total reactions: {}", self.reactions)?;

        if self.messages > 0 {
            let avg_length = self.message_lengths.0 / self.messages;
            writeln!(f, "\n📏 Message Lengths:")?;
            writeln!(f, "- Average: {} chars", avg_length)?;
            writeln!(f, "- Longest: {} chars", self.message_lengths.1)?;
        }

        if !self.participants.is_empty() {
            writeln!(f, "\n👥 Participants ({}):", self.participants.len())?;
            let mut participants: Vec<_> = self.participants.iter().collect();
            participants.sort_by(|a, b| b.1.cmp(a.1));

            for (i, (name, count)) in participants.iter().take(5).enumerate() {
                writeln!(f, "{}. {}: {}", i + 1, name, count)?;
            }
            if participants.len() > 5 {
                writeln!(f, "... and {} more", participants.len() - 5)?;
            }
        }

        if !self.text_entity_types.is_empty() {
            writeln!(
                f,
                "\n🔤 Text Entity Types ({}):",
                self.text_entity_types.len()
            )?;
            let mut entities: Vec<_> = self.text_entity_types.iter().collect();
            entities.sort_by(|a, b| b.1.cmp(a.1));

            for (entity_type, count) in entities {
                writeln!(f, "- {:15}: {:>4}", entity_type, count)?;
            }
        }

        Ok(())
    }
}
