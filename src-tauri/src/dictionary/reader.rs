use anyhow::{anyhow, Result};
use readmdict::Mdx;
use std::collections::HashMap;
use std::path::Path;

pub struct Dictionary {
    entries: HashMap<String, String>,
}

impl Dictionary {
    #[cfg(test)]
    pub fn from_entries(entries: HashMap<String, String>) -> Self {
        Dictionary { entries }
    }

    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("Dictionary file not found: {:?}", path));
        }

        let path_str = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;

        // Open MDX file: path, encoding (None = auto), substyle (true), passcode (None)
        let mdx = Mdx::new(path_str, None, true, None)
            .map_err(|e| anyhow!("Failed to open MDX file: {}", e))?;

        // Load all entries into a HashMap for fast lookup
        let entries_vec = mdx.items().map_err(|e| anyhow!("Failed to read items: {}", e))?;

        let mut entries = HashMap::new();
        for (key_bytes, value_bytes) in entries_vec {
            // Convert key to string (word)
            if let Ok(key) = String::from_utf8(key_bytes) {
                // Convert value to string (definition)
                if let Ok(value) = String::from_utf8(value_bytes) {
                    // Store with lowercase key for case-insensitive lookup
                    entries.insert(key.to_lowercase(), value);
                }
            }
        }

        Ok(Dictionary { entries })
    }

    /// Lookup a word in the dictionary.
    /// Returns the definition HTML/text if found.
    pub fn lookup(&self, word: &str) -> Option<String> {
        // Normalize the word to lowercase for case-insensitive lookup
        let normalized_word = word.to_lowercase().trim().to_string();

        self.entries.get(&normalized_word).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_dict_not_exists() {
        let result = Dictionary::open(Path::new("/nonexistent/path.mdx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let mut entries = HashMap::new();
        entries.insert("hello".to_string(), "你好".to_string());
        let dict = Dictionary::from_entries(entries);

        assert_eq!(dict.lookup("hello"), Some("你好".to_string()));
        assert_eq!(dict.lookup("Hello"), Some("你好".to_string()));
        assert_eq!(dict.lookup("HELLO"), Some("你好".to_string()));
    }

    #[test]
    fn test_lookup_trims_whitespace() {
        let mut entries = HashMap::new();
        entries.insert("hello".to_string(), "你好".to_string());
        let dict = Dictionary::from_entries(entries);

        assert_eq!(dict.lookup("  hello  "), Some("你好".to_string()));
        assert_eq!(dict.lookup("\thello\n"), Some("你好".to_string()));
    }

    #[test]
    fn test_lookup_nonexistent_returns_none() {
        let mut entries = HashMap::new();
        entries.insert("hello".to_string(), "你好".to_string());
        let dict = Dictionary::from_entries(entries);

        assert_eq!(dict.lookup("goodbye"), None);
        assert_eq!(dict.lookup(""), None);
    }
}
