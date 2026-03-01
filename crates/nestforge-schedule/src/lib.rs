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

#[derive(Clone)]
enum ScheduledTask {
    Interval {
        every: Duration,
        task: Arc<TaskFn>,
    },
    Timeout {
        after: Duration,
        task: Arc<TaskFn>,
    },
}

#[derive(Clone, Default)]
pub struct ScheduleRegistry {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl ScheduleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn every<F, Fut>(&self, every: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.tasks
            .lock()
            .expect("schedule registry should be writable")
            .push(ScheduledTask::Interval {
                every,
                task: Arc::new(move || Box::pin(task())),
            });
    }

    pub fn after<F, Fut>(&self, after: Duration, task: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.tasks
            .lock()
            .expect("schedule registry should be writable")
            .push(ScheduledTask::Timeout {
                after,
                task: Arc::new(move || Box::pin(task())),
            });
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
                ScheduledTask::Interval { every, task } => tokio::spawn(async move {
                    let mut interval = tokio::time::interval(every);
                    loop {
                        interval.tick().await;
                        (task)().await;
                    }
                }),
                ScheduledTask::Timeout { after, task } => tokio::spawn(async move {
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
}
