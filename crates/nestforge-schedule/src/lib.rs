use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use nestforge_core::Container;

type TaskFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
type TaskFn = dyn Fn() -> TaskFuture + Send + Sync + 'static;

/**
 * Internal representation of a scheduled task.
 */
#[derive(Clone)]
enum ScheduledTask {
    /** A task that runs at regular intervals */
    Interval {
        name: String,
        every: Duration,
        task: Arc<TaskFn>,
    },
    /** A task that runs once after a delay */
    Timeout {
        name: String,
        after: Duration,
        task: Arc<TaskFn>,
    },
}

/**
 * ScheduledJobKind
 *
 * The type of scheduled job.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledJobKind {
    /** A recurring job that runs at intervals */
    Interval,
    /** A one-time job that runs after a delay */
    Timeout,
}

/**
 * ScheduledJob
 *
 * Represents metadata about a scheduled job.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledJob {
    /** The name of the job */
    pub name: String,
    /** The kind of job (interval or timeout) */
    pub kind: ScheduledJobKind,
    /** The delay/interval duration */
    pub delay: Duration,
}

/**
 * ScheduleRegistry
 *
 * A registry for managing scheduled tasks in NestForge.
 * Allows registering interval and timeout-based tasks.
 *
 * # Usage
 * ```rust
 * let registry = ScheduleRegistry::new();
 * registry.every(Duration::from_secs(60), || async {
 *     println!("Running every minute");
 * });
 * ```
 */
#[derive(Clone, Default)]
pub struct ScheduleRegistry {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl ScheduleRegistry {
    /**
     * Creates a new empty ScheduleRegistry.
     */
    pub fn new() -> Self {
        Self::default()
    }

    /**
     * Creates a new ScheduleRegistryBuilder.
     */
    pub fn builder() -> ScheduleRegistryBuilder {
        ScheduleRegistryBuilder::new()
    }

    /**
     * Schedules a task to run at the specified interval.
     *
     * # Arguments
     * - `every`: The interval duration
     * - `task`: The async task to run
     */
    pub fn every<F, Fut>(&self, every: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.every_named(format!("interval:{every:?}"), every, task);
    }

    /**
     * Schedules a named task to run at the specified interval.
     */
    pub fn every_named<F, Fut>(&self, name: impl Into<String>, every: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.tasks
            .lock()
            .expect("schedule registry should be writable")
            .push(ScheduledTask::Interval {
                name: name.into(),
                every,
                task: Arc::new(move || Box::pin(task())),
            });
    }

    /**
     * Schedules a task to run once after the specified duration.
     */
    pub fn after<F, Fut>(&self, after: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.after_named(format!("timeout:{after:?}"), after, task);
    }

    pub fn after_named<F, Fut>(&self, name: impl Into<String>, after: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.tasks
            .lock()
            .expect("schedule registry should be writable")
            .push(ScheduledTask::Timeout {
                name: name.into(),
                after,
                task: Arc::new(move || Box::pin(task())),
            });
    }

    pub fn jobs(&self) -> Vec<ScheduledJob> {
        self.tasks
            .lock()
            .expect("schedule registry should be readable")
            .iter()
            .map(|task| match task {
                ScheduledTask::Interval { name, every, .. } => ScheduledJob {
                    name: name.clone(),
                    kind: ScheduledJobKind::Interval,
                    delay: *every,
                },
                ScheduledTask::Timeout { name, after, .. } => ScheduledJob {
                    name: name.clone(),
                    kind: ScheduledJobKind::Timeout,
                    delay: *after,
                },
            })
            .collect()
    }

    pub fn start(&self) {
        let tasks = self
            .tasks
            .lock()
            .expect("schedule registry should be readable")
            .clone();
        let mut handles = self
            .handles
            .lock()
            .expect("schedule handles should be writable");

        if !handles.is_empty() {
            return;
        }

        for task in tasks {
            let handle = match task {
                ScheduledTask::Interval {
                    name: _,
                    every,
                    task,
                } => tokio::spawn(async move {
                    let mut interval = tokio::time::interval(every);
                    loop {
                        interval.tick().await;
                        (task)().await;
                    }
                }),
                ScheduledTask::Timeout {
                    name: _,
                    after,
                    task,
                } => tokio::spawn(async move {
                    tokio::time::sleep(after).await;
                    (task)().await;
                }),
            };
            handles.push(handle);
        }
    }

    pub fn shutdown(&self) {
        let mut handles = self
            .handles
            .lock()
            .expect("schedule handles should be writable");
        for handle in handles.drain(..) {
            handle.abort();
        }
    }
}

pub fn start_schedules(container: &Container) -> Result<()> {
    let registry = container.resolve::<ScheduleRegistry>()?;
    registry.start();
    Ok(())
}

pub fn shutdown_schedules(container: &Container) -> Result<()> {
    let registry = container.resolve::<ScheduleRegistry>()?;
    registry.shutdown();
    Ok(())
}

#[derive(Default)]
pub struct ScheduleRegistryBuilder {
    registry: ScheduleRegistry,
}

impl ScheduleRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn every<F, Fut>(self, every: Duration, task: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.every_named(format!("interval:{every:?}"), every, task)
    }

    pub fn every_named<F, Fut>(self, name: impl Into<String>, every: Duration, task: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.registry.every_named(name, every, task);
        self
    }

    pub fn after<F, Fut>(self, after: Duration, task: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.after_named(format!("timeout:{after:?}"), after, task)
    }

    pub fn after_named<F, Fut>(self, name: impl Into<String>, after: Duration, task: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.registry.after_named(name, after, task);
        self
    }

    pub fn build(self) -> ScheduleRegistry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::*;

    #[tokio::test]
    async fn registry_starts_and_stops_interval_tasks() {
        let registry = ScheduleRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_for_task = Arc::clone(&counter);

        registry.every(Duration::from_millis(10), move || {
            let counter = Arc::clone(&counter_for_task);
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });

        registry.start();
        tokio::time::sleep(Duration::from_millis(35)).await;
        registry.shutdown();

        assert!(counter.load(Ordering::Relaxed) >= 1);
    }

    #[test]
    fn registry_tracks_named_jobs() {
        let registry = ScheduleRegistry::builder()
            .every_named("metrics", Duration::from_secs(30), || async {})
            .after_named("warmup", Duration::from_secs(5), || async {})
            .build();

        assert_eq!(
            registry.jobs(),
            vec![
                ScheduledJob {
                    name: "metrics".to_string(),
                    kind: ScheduledJobKind::Interval,
                    delay: Duration::from_secs(30),
                },
                ScheduledJob {
                    name: "warmup".to_string(),
                    kind: ScheduledJobKind::Timeout,
                    delay: Duration::from_secs(5),
                },
            ]
        );
    }
}
