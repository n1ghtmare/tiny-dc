use std::{
    fs::{DirEntry, ReadDir},
    path::PathBuf,
};

use ratatui::{prelude::*, widgets::*};

use crate::hotkeys::KeyCombo;

#[derive(Debug, PartialEq)]
pub enum EntryKind {
    File { extension: Option<String> },
    Directory,
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
    pub name: String,
}

impl TryFrom<DirEntry> for Entry {
    type Error = anyhow::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        Entry::try_from(value.path())
    }
}

impl TryFrom<PathBuf> for Entry {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file_type = value.metadata()?.file_type();
        let name = value
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let item = if file_type.is_dir() {
            Entry {
                path: value,
                kind: EntryKind::Directory,
                name,
            }
        } else {
            let extension = value.extension().map(|x| x.to_string_lossy().into_owned());

            Entry {
                path: value,
                kind: EntryKind::File { extension },
                name,
            }
        };

        Ok(item)
    }
}

/// This struct represents the data that will be used to render an entry in the list. It is used in
/// conjunction with the search query to determine how to render the entry.
///
/// It holds the prefix, search hit and suffix of the entry name, the next character after the
/// search hit, the kind of the entry and the shortcut assigned to the entry.
///
/// This allows us to render the entry in the UI with the search hit underlined and the shortcut
/// displayed next to the entry.
///
/// For example, if the entry name is "Cargo.toml" and the search query is "ar", the prefix will be
/// "C", the search hit will be "ar", the suffix will be "go.toml", the next character will be "g"
/// (the character immediately after the search hit)
///
/// The shortcut is assigned at a later stage and is used to quickly jump to the entry.
#[derive(Debug, PartialEq)]
pub struct EntryRenderData<'a> {
    prefix: &'a str,
    search_hit: &'a str,
    suffix: &'a str,

    /// The character that shouldn't appear in a hotkey sequence for the entry. That's normally the
    /// first character of the name or first character after the search hit. The idea is to allow
    /// the user to be able finish writing out the entry name without jumping to the entry itself.
    ///
    /// NOTE: that the character is converted to lowercase before being stored, since our search is
    /// case insensitive.
    pub illegal_char_for_hotkey: Option<char>,

    /// The kind of the entry, we need to keep track of this because we render directories
    /// differently than files.
    pub kind: &'a EntryKind,
    /// The key combo sequence assigned to the entry, it's an optional sequence of key combos.
    pub key_combo_sequence: Option<Vec<KeyCombo>>,
}

impl EntryRenderData<'_> {
    pub fn from_entry<T: AsRef<str>>(entry: &Entry, search_query: T) -> EntryRenderData<'_> {
        // Since our "search"/"filter" is case insensitive, and our for entries are always in lower
        // case, we need to make sure that the character we use for `illegal_char_for_hotkey` is
        // lowercase as well
        fn get_next_char_lowercase(name: &str) -> Option<char> {
            name.chars().next().and_then(|c| c.to_lowercase().next())
        }

        if search_query.as_ref().is_empty() {
            return EntryRenderData {
                prefix: &entry.name,
                search_hit: "",
                suffix: "",
                illegal_char_for_hotkey: get_next_char_lowercase(&entry.name),
                kind: &entry.kind,
                key_combo_sequence: None,
            };
        }

        let search_query = search_query.as_ref();
        let name = entry.name.to_lowercase();
        let search_query = search_query.to_lowercase();

        if let Some(index) = name.find(&search_query) {
            let prefix = &entry.name[..index];
            let search_hit = &entry.name[index..(index + search_query.len())];
            let suffix = &entry.name[(index + search_query.len())..];

            EntryRenderData {
                prefix,
                search_hit,
                suffix,
                illegal_char_for_hotkey: get_next_char_lowercase(suffix),
                kind: &entry.kind,
                key_combo_sequence: None,
            }
        } else {
            EntryRenderData {
                prefix: &entry.name,
                search_hit: "",
                suffix: "",
                illegal_char_for_hotkey: get_next_char_lowercase(&entry.name),
                kind: &entry.kind,
                key_combo_sequence: None,
            }
        }
    }
}

impl<'a> From<EntryRenderData<'a>> for ListItem<'a> {
    fn from(value: EntryRenderData<'a>) -> Self {
        let mut spans: Vec<Span> = Vec::new();

        // we want to display the search hit with underscore
        spans.push(Span::raw(value.prefix));
        spans.push(Span::styled(
            value.search_hit,
            Style::default().underlined(),
        ));
        spans.push(Span::raw(value.suffix));

        if value.kind == &EntryKind::Directory {
            spans.push(Span::raw("/"));

            if let Some(key_combo_sequence) = value.key_combo_sequence {
                spans.push(Span::raw("  ").style(Style::default().dark_gray()));
                for key_combo in key_combo_sequence {
                    spans.push(Span::styled(
                        key_combo.key_code.to_string(),
                        Style::default().black().on_green(),
                    ));
                }
            }

            let line = Line::from(spans);
            let style = Style::new().bold().fg(Color::White);

            ListItem::new(line).style(style)
        } else {
            let style = Style::new().dark_gray();
            let k = Line::from(spans);
            ListItem::new(k).style(style)
        }
    }
}

#[derive(Debug, Default)]
pub struct EntryList {
    pub items: Vec<Entry>,
    pub filtered_indices: Option<Vec<usize>>,
}

impl EntryList {
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get_filtered_entries(&self) -> Vec<&Entry> {
        match &self.filtered_indices {
            Some(indices) => indices.iter().map(|&i| &self.items[i]).collect(),
            None => self.items.iter().collect(),
        }
    }

    pub fn update_filtered_indices<T: AsRef<str>>(&mut self, value: T) {
        let value = value.as_ref().to_lowercase();

        if value.is_empty() {
            self.filtered_indices = None;
        } else {
            let indices = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, entry)| {
                    if entry.name.to_lowercase().contains(&value) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            self.filtered_indices = Some(indices);
        }
    }
}

impl TryFrom<ReadDir> for EntryList {
    type Error = anyhow::Error;

    fn try_from(value: ReadDir) -> Result<Self, Self::Error> {
        let mut items = Vec::new();

        for dir_entry_result in value.into_iter() {
            let dir_entry = dir_entry_result?;
            let result = Entry::try_from(dir_entry);

            match result {
                Ok(item) => items.push(item),
                Err(_) => {
                    // Skip the entry if it can't be converted to an Entry
                    //
                    // This is useful for example when the entry is a symlink to a file that
                    // doesn't exist
                    continue;
                }
            }
        }

        Ok(EntryList {
            items,
            ..Default::default()
        })
    }
}

impl TryFrom<Vec<PathBuf>> for EntryList {
    type Error = anyhow::Error;

    fn try_from(value: Vec<PathBuf>) -> Result<Self, Self::Error> {
        let mut items = Vec::new();

        for path in value {
            let result = Entry::try_from(path);
            match result {
                Ok(item) => items.push(item),
                Err(_) => {
                    // Skip the entry if it can't be converted to an Entry
                    //
                    // This is useful for example when the entry is a symlink to a file that
                    // doesn't exist
                    continue;
                }
            }
        }

        Ok(EntryList {
            items,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod entry_render_data {
        use super::*;

        #[test]
        fn entry_render_data_from_entry_works_correctly_with_search_query() {
            let entry = Entry {
                name: "Cargo.toml".into(),
                kind: EntryKind::File {
                    extension: Some("toml".into()),
                },
                path: PathBuf::from("/home/user/Cargo.toml"),
            };

            let entry_render_data: EntryRenderData = EntryRenderData::from_entry(&entry, "car");

            assert_eq!(
                entry_render_data,
                EntryRenderData {
                    prefix: "",
                    search_hit: "Car",
                    suffix: "go.toml",
                    illegal_char_for_hotkey: Some('g'),
                    kind: &EntryKind::File {
                        extension: Some("toml".into())
                    },
                    key_combo_sequence: None,
                }
            );

            let entry_render_data: EntryRenderData = EntryRenderData::from_entry(&entry, "toml");

            assert_eq!(
                entry_render_data,
                EntryRenderData {
                    prefix: "Cargo.",
                    search_hit: "toml",
                    suffix: "",
                    illegal_char_for_hotkey: None,
                    kind: &EntryKind::File {
                        extension: Some("toml".into())
                    },
                    key_combo_sequence: None,
                }
            );

            let entry_render_data: EntryRenderData = EntryRenderData::from_entry(&entry, "argo");

            assert_eq!(
                entry_render_data,
                EntryRenderData {
                    prefix: "C",
                    search_hit: "argo",
                    suffix: ".toml",
                    illegal_char_for_hotkey: Some('.'),
                    kind: &EntryKind::File {
                        extension: Some("toml".into())
                    },
                    key_combo_sequence: None,
                }
            );

            let entry_render_data: EntryRenderData = EntryRenderData::from_entry(&entry, "");

            assert_eq!(
                entry_render_data,
                EntryRenderData {
                    prefix: "Cargo.toml",
                    search_hit: "",
                    suffix: "",
                    illegal_char_for_hotkey: Some('c'),
                    kind: &EntryKind::File {
                        extension: Some("toml".into())
                    },
                    key_combo_sequence: None,
                }
            );
        }
    }
}
