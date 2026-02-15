//! Memory management for agent state and conversation history

use crate::error::Result;
use crate::error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Local;

/// Memory entry
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Memory manager for long-term and daily memory
pub struct MemoryManager {
    workspace: PathBuf,
    #[allow(dead_code)]
    max_size_mb: usize,
    memory_cache: Arc<RwLock<std::collections::HashMap<String, MemoryEntry>>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(workspace: impl AsRef<Path>, max_size_mb: usize) -> Result<Self> {
        let workspace = workspace.as_ref();
        let memory_dir = workspace.join("memory");
        fs::create_dir_all(&memory_dir)
            .map_err(|e| Error::internal(format!("Failed to create memory directory: {}", e)))?;

        Ok(MemoryManager {
            workspace: memory_dir,
            max_size_mb,
            memory_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Get long-term memory
    pub async fn get_long_term_memory(&self) -> Result<String> {
        let memory_file = self.workspace.join("MEMORY.md");
        if memory_file.exists() {
            fs::read_to_string(&memory_file)
                .map_err(|e| Error::internal(format!("Failed to read memory: {}", e)))
        } else {
            Ok(String::new())
        }
    }

    /// Update long-term memory
    pub async fn update_long_term_memory(&self, content: String) -> Result<()> {
        let memory_file = self.workspace.join("MEMORY.md");
        fs::write(&memory_file, content)
            .map_err(|e| Error::internal(format!("Failed to write memory: {}", e)))?;
        Ok(())
    }

    /// Get daily notes for today
    pub async fn get_daily_notes(&self) -> Result<String> {
        let today = Local::now();
        let date_dir = self.workspace.join(today.format("%Y%m").to_string());
        let date_file = date_dir.join(format!("{}.md", today.format("%Y%m%d")));

        if date_file.exists() {
            fs::read_to_string(&date_file)
                .map_err(|e| Error::internal(format!("Failed to read daily notes: {}", e)))
        } else {
            Ok(String::new())
        }
    }

    /// Update daily notes
    pub async fn update_daily_notes(&self, content: String) -> Result<()> {
        let today = Local::now();
        let date_dir = self.workspace.join(today.format("%Y%m").to_string());
        fs::create_dir_all(&date_dir)
            .map_err(|e| Error::internal(format!("Failed to create date directory: {}", e)))?;

        let date_file = date_dir.join(format!("{}.md", today.format("%Y%m%d")));
        fs::write(&date_file, content)
            .map_err(|e| Error::internal(format!("Failed to write daily notes: {}", e)))?;
        Ok(())
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> usize {
        // TODO: Implement actual memory usage tracking
        0
    }

    /// Store a memory entry
    pub async fn store_entry(&self, key: String, content: String) -> Result<()> {
        let now = Local::now().to_rfc3339();
        let entry = MemoryEntry {
            key: key.clone(),
            content,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut cache = self.memory_cache.write().await;
        cache.insert(key, entry);
        Ok(())
    }

    /// Retrieve a memory entry
    pub async fn retrieve_entry(&self, key: &str) -> Result<Option<MemoryEntry>> {
        let cache = self.memory_cache.read().await;
        Ok(cache.get(key).cloned())
    }
}
