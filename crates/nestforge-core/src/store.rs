use std::sync::{Arc, RwLock};

/*
Identifiable = tiny trait for entities that have an id
This lets the framework store handle ID assignment and lookups.
*/
pub trait Identifiable {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
}

/*
InMemoryStore<T> = framework-provided shared in-memory storage.

This hides Arc/RwLock and gives clean CRUD-ish methods for simple apps,
examples, and CLI scaffolds.
*/
#[derive(Clone)]
pub struct InMemoryStore<T> {
    items: Arc<RwLock<Vec<T>>>,
}

impl<T> Default for InMemoryStore<T>
where
    T: Identifiable + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InMemoryStore<T>
where
    T: Identifiable + Clone,
{
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_seed(seed: Vec<T>) -> Self {
        Self {
            items: Arc::new(RwLock::new(seed)),
        }
    }

    pub fn find_all(&self) -> Vec<T> {
        self.items.read().map(|items| items.clone()).unwrap_or_default()
    }

    pub fn find_by_id(&self, id: u64) -> Option<T> {
        self.items
            .read()
            .ok()
            .and_then(|items| items.iter().find(|item| item.id() == id).cloned())
    }

    /*
    create():
    - auto-generates next id
    - stores item
    - returns stored item
    */
    pub fn create(&self, mut item: T) -> T {
        let mut items = self.items.write().expect("store write lock poisoned");

        let next_id = items.iter().map(|item| item.id()).max().unwrap_or(0) + 1;
        item.set_id(next_id);

        items.push(item.clone());
        item
    }

    /*
    update_by_id():
    - mutates item in place via closure
    - returns updated item
    */
    pub fn update_by_id<F>(&self, id: u64, mut updater: F) -> Option<T>
    where
        F: FnMut(&mut T),
    {
        let mut items = self.items.write().ok()?;
        let item = items.iter_mut().find(|item| item.id() == id)?;

        updater(item);
        Some(item.clone())
    }
}
