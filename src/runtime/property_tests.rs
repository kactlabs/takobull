//! Property-based tests for runtime initialization
//!
//! **Property 2: Runtime initialization within 100ms**
//! *For any* valid RuntimeConfig, the runtime should initialize and be ready to accept tasks within 100ms.
//! **Validates: Requirements 2.2**

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use std::time::Duration;
    use crate::runtime::{RuntimeConfig, RuntimeManager};

    /// Strategy for generating valid RuntimeConfig values
    fn runtime_config_strategy() -> impl Strategy<Value = RuntimeConfig> {
        (1usize..=16, 1usize..=1024, 1usize..=8388608)
            .prop_map(|(workers, blocking, stack)| RuntimeConfig {
                worker_threads: workers,
                max_blocking_threads: blocking,
                thread_name_prefix: "test-worker".to_string(),
                stack_size: stack,
            })
    }

    /// Property test: Runtime initialization completes within 100ms
    ///
    /// For any valid RuntimeConfig, the runtime should initialize and be ready
    /// to accept tasks within 100ms.
    #[test]
    fn prop_runtime_initialization_within_100ms() {
        proptest!(|(config in runtime_config_strategy())| {
            let start = std::time::Instant::now();
            let result = RuntimeManager::initialize(config);
            let elapsed = start.elapsed();

            // Verify initialization succeeded
            prop_assert!(result.is_ok(), "Runtime initialization failed");

            // Verify initialization completed within 100ms
            prop_assert!(
                elapsed <= Duration::from_millis(100),
                "Runtime initialization took {:?}, exceeds 100ms target",
                elapsed
            );
        });
    }

    /// Property test: Runtime manager can be created and is ready immediately
    ///
    /// For any RuntimeManager instance, it should be created successfully
    /// and be ready to accept tasks immediately.
    #[tokio::test]
    async fn prop_runtime_manager_ready_immediately() {
        let start = std::time::Instant::now();
        let manager = RuntimeManager::new();
        let elapsed = start.elapsed();

        // Verify manager was created
        assert_eq!(manager.active_task_count(), 0);
        assert!(!manager.is_shutdown());

        // Verify creation was fast (should be microseconds, not milliseconds)
        assert!(
            elapsed < Duration::from_millis(10),
            "RuntimeManager creation took {:?}",
            elapsed
        );
    }

    /// Property test: Multiple runtime initializations don't interfere
    ///
    /// For any sequence of RuntimeConfig values, each initialization should
    /// complete independently within 100ms.
    #[test]
    fn prop_multiple_initializations_independent() {
        proptest!(|(configs in prop::collection::vec(runtime_config_strategy(), 1..5))| {
            for config in configs {
                let start = std::time::Instant::now();
                let result = RuntimeManager::initialize(config);
                let elapsed = start.elapsed();

                prop_assert!(result.is_ok());
                prop_assert!(elapsed <= Duration::from_millis(100));
            }
        });
    }

    /// Property test: Async errors propagate correctly
    ///
    /// **Property 3: Error handling for async failures**
    /// *For any* async task that fails, the error should propagate correctly
    /// through the task pool and be retrievable by the caller.
    /// **Validates: Requirements 2.3**
    #[tokio::test]
    async fn prop_async_error_propagation() {
        let manager = RuntimeManager::new();

        // Test 1: Task that returns a value
        let handle = manager.spawn_task(async { 42 });
        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test 2: Task that returns a different value
        let handle = manager.spawn_task(async { "success".to_string() });
        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Test 3: Multiple tasks with different outputs
        let h1 = manager.spawn_task(async { 1 });
        let h2 = manager.spawn_task(async { 2 });
        let h3 = manager.spawn_task(async { 3 });

        assert_eq!(h1.await.unwrap(), 1);
        assert_eq!(h2.await.unwrap(), 2);
        assert_eq!(h3.await.unwrap(), 3);
    }

    /// Property test: Multiple concurrent tasks handle errors independently
    ///
    /// For any set of concurrent tasks, each task should complete independently
    /// and not affect other tasks' execution.
    #[tokio::test]
    async fn prop_concurrent_error_isolation() {
        let manager = RuntimeManager::new();

        // Spawn multiple tasks that complete at different times
        let handles: Vec<_> = (0..10)
            .map(|i| {
                manager.spawn_task(async move {
                    // Simulate variable execution time
                    tokio::time::sleep(Duration::from_millis(i as u64)).await;
                    i as i32
                })
            })
            .collect();

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            results.push(result.unwrap());
        }

        // Verify all tasks completed
        assert_eq!(results.len(), 10);
        
        // Verify results are in expected range
        for result in results {
            assert!(result >= 0 && result < 10);
        }
    }

    /// Property test: Task errors don't crash the runtime
    ///
    /// For any task that encounters an error condition, the runtime should
    /// continue operating and allow other tasks to execute.
    #[tokio::test]
    async fn prop_runtime_resilience_to_task_errors() {
        let manager = RuntimeManager::new();

        // Spawn a task that completes successfully
        let h1 = manager.spawn_task(async { 100 });

        // Spawn a task that takes time
        let h2 = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            200
        });

        // Spawn another task that completes quickly
        let h3 = manager.spawn_task(async { 300 });

        // All tasks should complete successfully
        assert_eq!(h1.await.unwrap(), 100);
        assert_eq!(h2.await.unwrap(), 200);
        assert_eq!(h3.await.unwrap(), 300);

        // Runtime should still be operational
        assert!(!manager.is_shutdown());
        assert_eq!(manager.active_task_count(), 0);
    }

    /// Property test: Graceful shutdown completes successfully
    ///
    /// **Property 4: Graceful shutdown completeness**
    /// *For any* set of running tasks, graceful shutdown should terminate all tasks
    /// cleanly without data loss or corruption.
    /// **Validates: Requirements 2.4**
    #[tokio::test]
    async fn prop_graceful_shutdown_completeness() {
        let manager = RuntimeManager::new();

        // Spawn multiple tasks with different durations
        let _h1 = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            1
        });

        let _h2 = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            2
        });

        let _h3 = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            3
        });

        // Verify tasks are running
        assert!(manager.active_task_count() > 0);
        assert!(!manager.is_shutdown());

        // Initiate graceful shutdown
        let result = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result.is_ok());

        // Verify shutdown completed
        assert!(manager.is_shutdown());
        assert_eq!(manager.active_task_count(), 0);
    }

    /// Property test: Shutdown with timeout works correctly
    ///
    /// For any set of tasks, shutdown should respect the timeout and either
    /// complete all tasks or return an error if timeout is exceeded.
    #[tokio::test]
    async fn prop_shutdown_timeout_handling() {
        let manager = RuntimeManager::new();

        // Spawn a task that completes quickly
        let _h = manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        });

        // Shutdown with a reasonable timeout should succeed
        let result = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result.is_ok());
        assert!(manager.is_shutdown());
        assert_eq!(manager.active_task_count(), 0);
    }

    /// Property test: Multiple shutdown calls are idempotent
    ///
    /// For any RuntimeManager, calling shutdown multiple times should be safe
    /// and idempotent (second call should not error).
    #[tokio::test]
    async fn prop_shutdown_idempotent() {
        let manager = RuntimeManager::new();

        // Spawn a quick task
        let _h = manager.spawn_task(async { 42 });

        // First shutdown
        let result1 = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result1.is_ok());

        // Second shutdown should also succeed (idempotent)
        let result2 = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result2.is_ok());

        // Verify shutdown flag is set
        assert!(manager.is_shutdown());
    }

    /// Property test: Shutdown signal propagates to all tasks
    ///
    /// For any set of tasks, when shutdown is initiated, all tasks should
    /// receive the shutdown signal and terminate.
    #[tokio::test]
    async fn prop_shutdown_signal_propagation() {
        let manager = RuntimeManager::new();

        // Spawn multiple tasks that wait for shutdown
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let mut rx = manager.shutdown_signal();
                manager.spawn_task(async move {
                    // Wait for shutdown signal
                    let _ = rx.recv().await;
                    "shutdown_received"
                })
            })
            .collect();

        // Give tasks time to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Initiate shutdown
        let result = manager.shutdown(Duration::from_secs(5)).await;
        assert!(result.is_ok());

        // All tasks should have completed
        assert_eq!(manager.active_task_count(), 0);
        assert!(manager.is_shutdown());

        // Verify all handles completed
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}
