use std::{
    collections::{HashMap, HashSet},
    fmt::{self},
};

use serde::Serialize;

use crate::{Message, Reaction};

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserStats {
    pub count: u64,
    pub total_chars: u64,
    pub max_chars: u64,
    /// Word statistics: (word, count)
    #[serde(skip)]
    pub words: HashMap<String, usize>,
    #[serde(skip)]
    pub received_reactions: HashMap<String, usize>,
}

impl UserStats {
    pub fn add_message(&mut self, message: &str, filter: &HashSet<String>) -> &mut Self {
        let len = message.chars().count() as u64;
        self.count += 1;
        self.total_chars += len;
        self.max_chars = len.max(self.max_chars);

        for word in message
            .to_lowercase()
            .split_whitespace()
            .filter(|w| !filter.contains(*w))
        {
            *self.words.entry(word.to_string()).or_insert(0) += 1;
        }
        self
    }

    pub fn add_reactions(&mut self, reactions: &[Reaction]) -> &mut Self {
        for reaction in reactions {
            let (emoji, count) = match reaction {
                Reaction::Emoji { emoji, count, .. } => (emoji, count),
                Reaction::CustomEmoji {
                    document_id, count, ..
                } => (document_id, count),
            };
            *self
                .received_reactions
                .entry(emoji.to_string())
                .or_insert(0) += count;
        }
        self
    }

    pub fn avg_chars(&self) -> u64 {
        self.total_chars.checked_div(self.count).unwrap_or(0)
    }

    pub fn top_words(&self, max: usize) -> Vec<(&String, &usize)> {
        let mut words: Vec<_> = self.words.iter().collect();
        words.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
        words.truncate(max);
        words
    }
}

impl std::iter::Sum for UserStats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UserStats::default(), |mut acc, item| {
            acc.count += item.count;
            acc.total_chars += item.total_chars;
            acc.max_chars = acc.max_chars.max(item.max_chars);
            for (word, count) in item.words {
                *acc.words.entry(word).or_insert(0) += count;
            }
            for (reaction, count) in item.received_reactions {
                *acc.received_reactions.entry(reaction).or_insert(0) += count;
            }
            acc
        })
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ChatStats {
    pub messages: u64,
    pub service_messages: u64,
    pub edited: u64,
    pub participants: HashMap<String, UserStats>,
    pub text_entity_types: HashMap<String, u64>,
    pub settings: StatsSettings,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct StatsSettings {
    /// How many most frequent words to display.
    pub max_words: usize,
    /// Wheter to show most frequent text entity types.
    pub show_entities: bool,
}

impl ChatStats {
    const HEAD_SIZE: usize = 5;

    pub fn analyze(&mut self, messages: &[Message]) {
        self.messages += messages.len() as u64;
        let words: HashSet<_> = stop_words::get(stop_words::LANGUAGE::Russian)
            .into_iter()
            .collect();

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
                    self.participants
                        .entry(from.clone())
                        .or_default()
                        .add_message(&text.to_string(), &words)
                        .add_reactions(reactions);

                    if edited.is_some() || edited_unixtime.is_some() {
                        self.edited += 1;
                    }

                    self.count_entities(text_entities);
                }
                Message::Service { text_entities, .. } => {
                    self.service_messages += 1;
                    self.count_entities(text_entities);
                }
            }
        }
    }

    fn count_entities(&mut self, entities: &[crate::TextEntity]) {
        for entity in entities {
            *self
                .text_entity_types
                .entry(entity.entity_type.clone())
                .or_default() += 1;
        }
    }

    fn display_user_stats(&self, stats: &UserStats, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if stats.count == 0 {
            return write!(f, "- No messages");
        }

        writeln!(f, "- Messages       : {}", stats.count)?;
        writeln!(f, "- Avg. length    : {} chars", stats.avg_chars())?;
        writeln!(f, "- Longest message: {} chars", stats.max_chars)?;

        let mut received: Vec<_> = stats.received_reactions.iter().collect();
        received.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
        let received = received
            .into_iter()
            .map(|(r, c)| format!("{r}×{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(f, "- Reactions      : {}", received)?;

        let top_words = stats.top_words(self.settings.max_words);
        if !top_words.is_empty() {
            let words_line = top_words
                .iter()
                .map(|(word, count)| format!("{} ({})", word, count))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(f, "- Top words      : {}", words_line)?;
        }
        Ok(())
    }
}

impl fmt::Display for ChatStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let combined = self.participants.values().cloned().sum::<UserStats>();
        let reactions: usize = combined.received_reactions.values().sum();

        writeln!(f, "📊 Chat Statistics Summary\n=========================")?;
        writeln!(f, "💬 Total messages     : {}", self.messages)?;
        writeln!(f, "⚙️ Service messages   : {}", self.service_messages)?;
        writeln!(f, "✏️ Edited messages    : {}", self.edited)?;
        writeln!(f, "❤️ Total reactions    : {reactions}",)?;

        if combined.count > 0 {
            writeln!(f, "\n📏 Combined Participant Stats:")?;
            self.display_user_stats(&combined, f)?;
        }

        if !self.participants.is_empty() {
            let mut participants: Vec<_> = self.participants.iter().collect();
            participants.sort_unstable_by_key(|(_, stats)| std::cmp::Reverse(stats.count));

            writeln!(f, "\n👥 Top Participants ({}):", participants.len())?;
            for (i, (name, stats)) in participants.iter().take(Self::HEAD_SIZE).enumerate() {
                let percent = 100.0 * (stats.total_chars as f64 / combined.total_chars as f64);
                writeln!(f, "\n{}. {name}  (Character share: {percent:.0}%)", i + 1)?;
                self.display_user_stats(&stats, f)?;
            }
            if participants.len() > Self::HEAD_SIZE {
                writeln!(f, "... and {} more", participants.len() - Self::HEAD_SIZE)?;
            }
        }

        if !self.text_entity_types.is_empty() && self.settings.show_entities {
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
