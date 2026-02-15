//! Async runtime management for TakoBull
//!
//! This module provides:
//! - Tokio async runtime initialization and configuration
//! - Graceful shutdown mechanism for all async tasks
//! - Task pool for managing concurrent operations
//! - Runtime metrics and monitoring

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};

/// Configuration for the async runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum number of worker threads
    pub worker_threads: usize,
    /// Maximum number of blocking threads
    pub max_blocking_threads: usize,
    /// Thread name prefix
    pub thread_name_prefix: String,
    /// Stack size for spawned tasks (in bytes)
    pub stack_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            max_blocking_threads: 512,
            thread_name_prefix: "takobull-worker".to_string(),
            stack_size: 2 * 1024 * 1024, // 2MB
        }
    }
}

/// Manages the async runtime and task lifecycle
pub struct RuntimeManager {
    /// Broadcast channel for shutdown signals
    shutdown_tx: broadcast::Sender<()>,
    /// Active task count
    active_tasks: Arc<AtomicUsize>,
    /// Shutdown flag
    is_shutdown: Arc<AtomicBool>,
}

impl RuntimeManager {
    /// Create a new runtime manager
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            shutdown_tx,
            active_tasks: Arc::new(AtomicUsize::new(0)),
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Initialize the tokio runtime with the given configuration
    ///
    /// # Requirements
    /// - Requirement 2.1: Uses tokio as the async runtime
    /// - Requirement 2.2: Runtime initializes and is ready within 100ms
    pub fn initialize(config: RuntimeConfig) -> Result<()> {
        debug!("Initializing async runtime with config: {:?}", config);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .max_blocking_threads(config.max_blocking_threads)
            .thread_name("tacobot-worker")
            .thread_stack_size(config.stack_size)
            .enable_all()
            .build()
            .map_err(|e| Error::runtime(format!("Failed to initialize tokio runtime: {}", e)))?;

        // Verify runtime is ready
        let start = std::time::Instant::now();
        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(1)).await;
        });
        let elapsed = start.elapsed();

        if elapsed > Duration::from_millis(100) {
            warn!(
                "Runtime initialization took {:?}, exceeds 100ms target",
                elapsed
            );
        } else {
            debug!("Runtime initialized in {:?}", elapsed);
        }

        info!(
            "Async runtime initialized: {} worker threads, {} max blocking threads",
            config.worker_threads, config.max_blocking_threads
        );

        Ok(())
    }

    /// Spawn a task on the runtime with shutdown signal support
    ///
    /// # Requirements
    /// - Requirement 2.5: Maintains a task pool for managing concurrent operations
    pub fn spawn_task<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Default + Send + 'static,
    {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        let active_tasks = Arc::clone(&self.active_tasks);
        let shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let result = tokio::select! {
                res = future => {
                    res
                }
                _ = Self::wait_for_shutdown(shutdown_rx) => {
                    debug!("Task interrupted by shutdown signal");
                    F::Output::default()
                }
            };
            active_tasks.fetch_sub(1, Ordering::SeqCst);
            result
        })
    }

    /// Get the number of active tasks
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Get a shutdown signal receiver
    pub fn shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Wait for shutdown signal
    async fn wait_for_shutdown(mut rx: broadcast::Receiver<()>) {
        let _ = rx.recv().await;
    }

    /// Initiate graceful shutdown
    ///
    /// # Requirements
    /// - Requirement 2.4: Supports graceful shutdown of all async tasks
    pub async fn shutdown(&self, timeout: Duration) -> Result<()> {
        if self.is_shutdown.swap(true, Ordering::SeqCst) {
            debug!("Shutdown already in progress");
            return Ok(());
        }

        info!("Initiating graceful shutdown");

        // Signal all tasks to shutdown
        if let Err(e) = self.shutdown_tx.send(()) {
            warn!("Failed to broadcast shutdown signal: {}", e);
        }

        // Wait for all tasks to complete with timeout
        let start = std::time::Instant::now();
        loop {
            let active = self.active_task_count();
            if active == 0 {
                info!("All tasks completed gracefully");
                break;
            }

            if start.elapsed() > timeout {
                error!(
                    "Shutdown timeout exceeded with {} active tasks remaining",
                    active
                );
                return Err(Error::runtime(format!(
                    "Graceful shutdown timeout: {} tasks still active",
                    active
                )));
            }

            debug!("Waiting for {} tasks to complete", active);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        info!("Graceful shutdown completed");
        Ok(())
    }

    /// Check if shutdown has been initiated
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        if !self.is_shutdown.load(Ordering::SeqCst) {
            warn!("RuntimeManager dropped without graceful shutdown");
        }
    }
}

/// Task pool for managing concurrent operations
///
/// # Requirements
/// - Requirement 2.5: Creates task pool for managing concurrent operations
pub struct TaskPool {
    manager: RuntimeManager,
    max_concurrent: usize,
}

impl TaskPool {
    /// Create a new task pool with the given maximum concurrent tasks
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            manager: RuntimeManager::new(),
            max_concurrent,
        }
    }

    /// Get the maximum number of concurrent tasks
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Get the current number of active tasks
    pub fn active_tasks(&self) -> usize {
        self.manager.active_task_count()
    }

    /// Check if the pool can accept more tasks
    pub fn can_accept_task(&self) -> bool {
        self.manager.active_task_count() < self.max_concurrent
    }

    /// Spawn a task on the pool
    pub fn spawn_task<F>(&self, future: F) -> Result<JoinHandle<F::Output>>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Default + Send + 'static,
    {
        if !self.can_accept_task() {
            return Err(Error::runtime(format!(
                "Task pool at capacity: {}/{}",
                self.active_tasks(),
                self.max_concurrent
            )));
        }

        Ok(self.manager.spawn_task(future))
    }

    /// Shutdown the task pool gracefully
    pub async fn shutdown(&self, timeout: Duration) -> Result<()> {
        self.manager.shutdown(timeout).await
    }
}

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_manager_creation() {
        let manager = RuntimeManager::new();
        assert_eq!(manager.active_task_count(), 0);
        assert!(!manager.is_shutdown());
    }

    #[tokio::test]
    async fn test_spawn_task() {
        let manager = RuntimeManager::new();
        let handle = manager.spawn_task(async { 42 });
        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let manager = RuntimeManager::new();

        // Spawn a task
        let _handle = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
        });

        assert_eq!(manager.active_task_count(), 1);

        // Shutdown
        let result = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result.is_ok());
        assert!(manager.is_shutdown());
    }

    #[tokio::test]
    async fn test_task_pool_creation() {
        let pool = TaskPool::new(10);
        assert_eq!(pool.max_concurrent(), 10);
        assert_eq!(pool.active_tasks(), 0);
        assert!(pool.can_accept_task());
    }

    #[tokio::test]
    async fn test_task_pool_capacity() {
        let pool = TaskPool::new(2);

        // Spawn tasks up to capacity
        let _h1 = pool.spawn_task(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
        let _h2 = pool.spawn_task(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        assert_eq!(pool.active_tasks(), 2);
        assert!(!pool.can_accept_task());

        // Try to spawn beyond capacity
        let result = pool.spawn_task(async { 42 });
        assert!(result.is_err());
    }
}
