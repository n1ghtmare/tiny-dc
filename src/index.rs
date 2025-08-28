use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    time::SystemTime,
};

pub const DEFAULT_INDEX_FILE_NAME: &str = ".tiny-dc";

#[derive(Debug)]
pub struct DirectoryIndexEntry {
    /// Combined score based on frequence and recency
    rank: f64,
    /// Unix timestamp of the last access
    last_accessed: u64,
}

impl DirectoryIndexEntry {
    fn new() -> Self {
        DirectoryIndexEntry {
            rank: 0.0,
            last_accessed: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.last_accessed = now;

        // Decay the previous rank slightly (1% decay) and add a fixed bonus for this new access.
        // The factor 0.99 is used to slowly forget old accesses, while adding 1.0 ensures each
        // access gives a boost.
        self.rank = (self.rank * 0.99) + 1.0;
    }

    fn frecent_score(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate the time since the last access
        let dx = now - self.last_accessed;

        // Calculate the frecent score, this was taken from rupa/z: https://github.com/rupa/z
        //
        // Breakdown of the scoring calculation:
        // - `0.0001 * dx`: Small increase per second of inactivity.
        // - `+ 1.0`: Ensures that when dx is zero, the term is 1.0, avoiding division by zero.
        // - `+ 0.25`: Additional adjustment to calibrate the decay effect.
        // - Division by this sum reduces the impact of the rank as time passes.
        // - Multiplication by 3.75 scales the effect.
        // - Finally, multiplying by 10000.0 amplifies the score to a more useful range.
        10000.0 * self.rank * (3.75 / ((0.0001 * dx as f64 + 1.0) + 0.25))
    }
}

/// A struct representing the directory index, which is a map of paths to their corresponding
/// `DirectoryIndexEntry` objects. The index is stored on disk in a file specified by the user (or
/// a default location see `DEFAULT_INDEX_FILE_NAME`).
#[derive(Debug, Default)]
pub struct DirectoryIndex {
    path: PathBuf,
    data: HashMap<PathBuf, DirectoryIndexEntry>,
}

impl DirectoryIndex {
    /// Reads the index from disk, if it doesn't exist, creates a new one
    pub fn try_from(path: PathBuf) -> anyhow::Result<Self> {
        let file = if path.exists() {
            // Open the file if it exists
            File::open(&path)?
        } else {
            // Create the file if it doesn't exist
            File::create_new(&path)?
        };

        let reader = BufReader::new(file);
        let mut data = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split('|').collect();

            if parts.len() != 3 {
                // Skip malformed lines
                continue;
            }

            let path = PathBuf::from(parts[0]);
            let rank: f64 = parts[1].parse().unwrap_or(0.0);
            let last_accessed: u64 = parts[2].parse().unwrap_or(0);

            let entry = DirectoryIndexEntry {
                last_accessed,
                rank,
            };
            data.insert(path.clone(), entry);
        }

        Ok(DirectoryIndex { path, data })
    }

    fn save_to_disk(&self) -> anyhow::Result<()> {
        // Save the index to disk
        let file = File::create(self.path.clone())?;
        let mut writer = BufWriter::new(file);

        for (path, entry) in &self.data {
            writeln!(
                writer,
                "{}|{}|{}",
                path.display(),
                entry.rank,
                entry.last_accessed
            )?;
        }

        Ok(())
    }

    /// Pushes a new path to the index and saves it to disk. If the path doesn't exist it's a
    /// no-op. If you push the same path multiple times, it will update the rank and last accessed
    /// time.
    pub fn push(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if !path.exists() {
            // If the path doesn't exist, we don't want to add it to the index
            return Ok(());
        }

        if let Some(entry) = self.data.get_mut(&path) {
            // Entry exists, update it (to update the score and last accessed time)
            entry.update();
        } else {
            let entry = DirectoryIndexEntry::new();
            self.data.insert(path, entry);
        }

        self.save_to_disk()?;

        Ok(())
    }

    /// Finds the top-ranked directory matching the query.
    ///
    /// If a non-existing path is found as a match, it will be removed from the index and the next
    /// match will be returned until the index is exhausted. The index will be updated if a removal
    /// occurs.
    ///
    /// The inner workings of this algo was heavily inspured by `rupa/z: https://github.com/rupa/z
    pub fn z(&mut self, queries: Vec<String>) -> anyhow::Result<Option<PathBuf>> {
        let mut matches = Vec::new();

        for (path, stats) in &self.data {
            let path_str = path.to_string_lossy();
            let frecent_score = stats.frecent_score();

            if queries.iter().all(|q| path_str.contains(q)) {
                // Higher priority: case-sensitive match.
                matches.push((path.clone(), frecent_score, 0))
            } else if queries.iter().all(|q| path_str.to_lowercase().contains(q)) {
                // Lower priority: case-insentitive match.
                matches.push((path.clone(), frecent_score, 1));
            }
        }

        if matches.is_empty() {
            return Ok(None);
        }

        // Look for a candidate that is an ancestor of every match.
        if let Some((ancestor, _, _)) = matches.iter().find(|(candidate, _, _)| {
            matches
                .iter()
                .all(|(other, _, _)| other.starts_with(candidate))
        }) {
            return Ok(Some(ancestor.clone()));
        }

        // Fallback: sort by match priority, frecent score (high to low), and then by fewer path
        // components.
        matches.sort_by(|a, b| {
            a.2.cmp(&b.2)
                .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.0.components().count().cmp(&b.0.components().count()))
        });

        let mut is_index_updated = false;
        let mut result = None;

        for (path, _, _) in matches.iter() {
            if path.exists() {
                // If the path exists, break and return it
                result = Some(path.clone());
                break;
            }

            // If the path doesn't exist, remove it from the index
            self.data.remove(path);
            is_index_updated = true;
        }

        if is_index_updated {
            // Save the index to disk if it was updated
            self.save_to_disk()?;
        }

        Ok(result)
    }

    /// Returns all entries in the index ordered by their frecent score.
    pub fn get_all_entries_ordered_by_rank(&self) -> Vec<PathBuf> {
        let mut entries: Vec<_> = self.data.iter().collect();
        entries.sort_by(|a, b| {
            b.1.frecent_score()
                .partial_cmp(&a.1.frecent_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.into_iter().map(|(path, _)| path.clone()).collect()
    }
}
