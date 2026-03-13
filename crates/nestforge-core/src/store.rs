use std::sync::{Arc, RwLock};

/**
 * Identifiable Trait
 *
 * A minimal trait for entities that have an ID field.
 * This enables the framework's in-memory store to handle ID assignment
 * and entity lookups consistently across different entity types.
 */
pub trait Identifiable {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
}

/**
 * In-Memory Store
 *
 * A framework-provided storage mechanism for simple applications, examples,
 * and CLI scaffolds. It provides a thread-safe, cloneable storage with
 * basic CRUD operations.
 *
 * The store uses Arc<RwLock<Vec<T>>> internally to allow concurrent reads
 * while providing exclusive writes. This design supports simple use cases
 * without requiring external database dependencies.
 *
 * # Type Parameters
 * - `T`: The entity type being stored, must implement `Identifiable` and `Clone`
 *
 * # Example
 * ```rust
 * #[derive(Clone)]
 * struct User { id: u64, name: String }
 * impl Identifiable for User { ... }
 *
 * let store = InMemoryStore::<User>::new();
 * let user = store.create(User { id: 0, name: "Alice".to_string() });
 * ```
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
    /**
     * Creates a new, empty in-memory store.
     */
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /**
     * Creates a store pre-populated with seed data.
     *
     * # Arguments
     * - `seed`: Initial collection of items to populate the store with
     */
    pub fn with_seed(seed: Vec<T>) -> Self {
        Self {
            items: Arc::new(RwLock::new(seed)),
        }
    }

    /**
     * Retrieves all items from the store.
     *
     * Returns a clone of all stored items wrapped in a Result for
     * safe handling of lock poisoning scenarios.
     */
    pub fn find_all(&self) -> Vec<T> {
        self.items
            .read()
            .map(|items| items.clone())
            .unwrap_or_default()
    }

    /**
     * Finds a single item by its ID.
     *
     * # Arguments
     * - `id`: The unique identifier to search for
     *
     * Returns `Option<T>` - the found item or None if not found.
     */
    pub fn find_by_id(&self, id: u64) -> Option<T> {
        self.items
            .read()
            .ok()
            .and_then(|items| items.iter().find(|item| item.id() == id).cloned())
    }

    /**
     * Returns the total count of items in the store.
     */
    pub fn count(&self) -> usize {
        self.items.read().map(|items| items.len()).unwrap_or(0)
    }

    /**
     * Creates a new item in the store.
     *
     * Automatically generates the next available ID by finding
     * the maximum ID in the current collection and incrementing by 1.
     * The item is then cloned and stored.
     *
     * # Arguments
     * - `item`: The item to create (ID field will be overwritten)
     *
     * Returns the stored item with the assigned ID.
     */
    pub fn create(&self, mut item: T) -> T {
        let mut items = self.items.write().expect("store write lock poisoned");

        let next_id = items.iter().map(|item| item.id()).max().unwrap_or(0) + 1;
        item.set_id(next_id);

        items.push(item.clone());
        item
    }

    /**
     * Updates an item in place using a closure.
     *
     * # Arguments
     * - `id`: The ID of the item to update
     * - `updater`: A closure that mutates the found item
     *
     * Returns the updated item or None if not found.
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

    /**
     * Replaces an item entirely by ID.
     *
     * # Arguments
     * - `id`: The ID of the item to replace
     * - `replacement`: The new item data
     *
     * Returns the replacement or None if not found.
     */
    pub fn replace_by_id(&self, id: u64, mut replacement: T) -> Option<T> {
        let mut items = self.items.write().ok()?;
        let existing = items.iter_mut().find(|item| item.id() == id)?;
        replacement.set_id(id);
        *existing = replacement;
        Some(existing.clone())
    }

    /**
     * Deletes an item from the store by ID.
     *
     * # Arguments
     * - `id`: The ID of the item to delete
     *
     * Returns the removed item or None if not found.
     */
    pub fn delete_by_id(&self, id: u64) -> Option<T> {
        let mut items = self.items.write().ok()?;
        let index = items.iter().position(|item| item.id() == id)?;
        Some(items.remove(index))
    }
}
