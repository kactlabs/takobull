//! Cron service for scheduled tasks

use crate::error::Result;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cron schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronSchedule {
    /// One-time execution at specific time (milliseconds since epoch)
    At(i64),
    /// Recurring execution every N milliseconds
    Every(i64),
    /// Cron expression (e.g., "0 0 * * *")
    Cron(String),
}

/// Cron job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronPayload {
    pub kind: String,
    pub message: String,
    pub command: Option<String>,
    pub deliver: bool,
    pub channel: Option<String>,
    pub to: Option<String>,
}

/// Cron job state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobState {
    pub next_run_at_ms: Option<i64>,
    pub last_run_at_ms: Option<i64>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
}

/// Cron job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub schedule: CronSchedule,
    pub payload: CronPayload,
    pub state: CronJobState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub delete_after_run: bool,
}

/// Cron service for managing scheduled tasks
pub struct CronService {
    jobs: Arc<RwLock<HashMap<String, CronJob>>>,
    store_path: Option<PathBuf>,
}

impl CronService {
    /// Create a new cron service
    pub fn new() -> Self {
        CronService {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            store_path: None,
        }
    }

    /// Create a new cron service with persistent storage
    pub fn with_storage(store_path: impl AsRef<Path>) -> Result<Self> {
        let path = store_path.as_ref();
        fs::create_dir_all(path)
            .map_err(|e| Error::internal(format!("Failed to create cron storage directory: {}", e)))?;

        let mut service = CronService {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            store_path: Some(path.to_path_buf()),
        };

        // Load existing jobs
        service.load_jobs()?;
        Ok(service)
    }

    /// Add a cron job
    pub async fn add_job(&self, mut job: CronJob) -> Result<String> {
        if job.id.is_empty() {
            job.id = Uuid::new_v4().to_string();
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        job.created_at_ms = now;
        job.updated_at_ms = now;

        let job_id = job.id.clone();
        let mut jobs = self.jobs.write().await;
        jobs.insert(job_id.clone(), job.clone());

        if let Some(path) = &self.store_path {
            self.save_job_to_disk(path, &job)?;
        }

        Ok(job_id)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<CronJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id)
            .cloned()
            .ok_or_else(|| Error::internal(format!("Job not found: {}", job_id)))
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Result<Vec<CronJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    /// Delete a job
    pub async fn delete_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(job_id);

        if let Some(path) = &self.store_path {
            let job_file = path.join(format!("{}.json", job_id));
            let _ = fs::remove_file(job_file);
        }

        Ok(())
    }

    // Private helpers

    fn save_job_to_disk(&self, base_path: &Path, job: &CronJob) -> Result<()> {
        let job_file = base_path.join(format!("{}.json", job.id));
        let json = serde_json::to_string_pretty(job)
            .map_err(|e| Error::serialization(format!("Failed to serialize job: {}", e)))?;

        fs::write(&job_file, json)
            .map_err(|e| Error::internal(format!("Failed to save job: {}", e)))?;

        Ok(())
    }

    fn load_jobs(&mut self) -> Result<()> {
        if let Some(path) = &self.store_path {
            if !path.exists() {
                return Ok(());
            }

            for entry in fs::read_dir(path)
                .map_err(|e| Error::internal(format!("Failed to read cron directory: {}", e)))?
            {
                let entry = entry
                    .map_err(|e| Error::internal(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();

                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(job) = serde_json::from_str::<CronJob>(&content) {
                            let mut jobs = futures::executor::block_on(self.jobs.write());
                            jobs.insert(job.id.clone(), job);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for CronService {
    fn default() -> Self {
        Self::new()
    }
}
