use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use directories_next::UserDirs;
use fs_err as fs;
use log::warn;
use serde::Deserialize;

/// Name of the directory under Downloads where Telegram exports live.
const TG_DIRECTORY_NAME: &str = "Telegram Desktop";
/// Filename inside each chat folder containing the JSON manifest.
const RESULT_FILE: &str = "result.json";

/// Holds all chats discovered under a Telegram export root.
pub struct Storage {
    #[allow(unused)] // Keep until it's clear whether `root` is needed externally
    root: PathBuf,
    /// Map from Telegram `chat_id` to its on‑disk `ChatFile`.
    pub chats: HashMap<i64, ChatFile>,
}

impl Storage {
    /// Locate `~/Downloads/Telegram Desktop` and load all `result.json` manifests.
    pub fn new() -> Result<Self, OpenError> {
        let root = UserDirs::new()
            .ok_or(OpenError::NoHome)?
            .download_dir()
            .ok_or(OpenError::NoDownload)?
            .join(TG_DIRECTORY_NAME);

        Self::from_path(root)
    }

    /// Load all chats from the given path (each subdirectory is expected
    /// to contain a `result.json`).
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, OpenError> {
        let chats = fs::read_dir(path.as_ref())
            .map_err(OpenError::NoTelegram)?
            // Skip entries we failed to read, warning on error
            .filter_map(|e| e.inspect_err(|e| warn!("skipping entry: {e}")).ok())
            .map(|entry| entry.path())
            .filter(|p| p.is_dir())
            .filter_map(try_load_chat)
            .collect();

        Ok(Self {
            root: path.as_ref().into(),
            chats,
        })
    }
}

/// Attempt to read `chat_dir/result.json` and deserialize it.
fn try_load_chat(chat_dir: PathBuf) -> Option<(i64, ChatFile)> {
    let manifest = chat_dir.join(RESULT_FILE);

    let bytes = fs::read(&manifest)
        .inspect_err(|e| warn!("couldn't read {:?}: {e}", manifest))
        .ok()?;

    let info = serde_json::from_slice::<ChatInfo>(&bytes)
        .inspect_err(|e| warn!("invalid JSON in {:?}: {e}", manifest))
        .ok()?;

    Some((
        info.id,
        ChatFile {
            path: manifest,
            info,
        },
    ))
}

/// A discovered chat file on disk: its path plus parsed metadata.
#[derive(Debug)]
pub struct ChatFile {
    /// Filesystem path to the `result.json` we loaded.
    pub path: PathBuf,
    /// Parsed chat metadata.
    pub info: ChatInfo,
}

/// A basic description of a Telegram chat, as found in `result.json`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ChatInfo {
    /// The display name of the chat
    pub name: String,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub id: i64,
}

/// An error that can occur when opening or reading Telegram exports.
#[derive(thiserror::Error, Debug)]
pub enum OpenError {
    /// Could not find a home directory on this OS.
    #[error("no valid home directory path could be retrieved from the operating system")]
    NoHome,

    /// Could not find a Downloads directory under the user’s home.
    #[error("no valid download directory path could be retrieved from the operating system")]
    NoDownload,

    /// The top‑level `Downloads/Telegram Desktop` directory could not be read.
    #[error("can't open Telegram output directory: {0}")]
    NoTelegram(io::Error),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{Storage, storage::ChatInfo};

    #[test]
    fn single() -> anyhow::Result<()> {
        let td_single = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("td_single");

        let storage = Storage::from_path(td_single)?;

        assert_eq!(1, storage.chats.len());
        let expected = ChatInfo {
            name: "Name Surname".to_string(),
            chat_type: "personal_chat".to_string(),
            id: 1,
        };
        let chat = storage.chats.get(&1).expect("chat 1 missing");
        assert_eq!(expected, chat.info);

        Ok(())
    }
}
