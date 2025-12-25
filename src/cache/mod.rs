use std::{
    array,
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crossbeam::utils::CachePadded;

pub struct Cache<T, const LEN: usize = 4>
where
    T: Clone,
{
    index: CachePadded<AtomicUsize>,
    writing: CachePadded<AtomicBool>,
    items: [Item<T>; LEN],
}

struct Item<T> {
    count: CachePadded<AtomicUsize>,
    data: UnsafeCell<Option<T>>,
}

// Safety: Cache is designed for concurrent access
// - UnsafeCell is only accessed through atomic guards (count for reads, writing for writes)
// - Reads increment/decrement count atomically around the UnsafeCell access
// - Writes hold the writing lock and check count is zero before accessing UnsafeCell
unsafe impl<T: Clone, const LEN: usize> Sync for Cache<T, LEN> {}

impl<T: Clone, const LEN: usize> Cache<T, LEN> {
    const CHECK_LEN_IS_POWER_OF_TWO: () = assert!(LEN.is_power_of_two() == true);
    const LEN_MASK: usize = LEN - 1;

    pub fn new(data: T) -> Self {
        let _ = Self::CHECK_LEN_IS_POWER_OF_TWO;

        let mut items = array::from_fn(|_| Item {
            count: CachePadded::new(AtomicUsize::new(0)),
            data: UnsafeCell::new(None),
        });

        *items[0].data.get_mut() = Some(data);

        Self {
            index: CachePadded::new(AtomicUsize::new(0)),
            writing: CachePadded::new(AtomicBool::new(false)),
            items,
        }
    }

    pub fn get_data(&self) -> T {
        let index = self.index();

        self.items[index].count.fetch_add(1, Ordering::Release);

        let data = unsafe {
            (*self.items[index].data.get())
                .as_ref()
                .unwrap_unchecked()
                .clone()
        };

        self.items[index].count.fetch_sub(1, Ordering::Release);

        data
    }

    pub fn update(&self, data: T) {
        while self.writing.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        let current_index = self.index.load(Ordering::Acquire);
        let mut next_index = current_index;

        loop {
            next_index = (next_index + 1) & Self::LEN_MASK;

            if next_index == current_index {
                continue;
            }

            let count = self.items[next_index].count.load(Ordering::Acquire);

            if count == 0 {
                break;
            }
        }

        unsafe {
            drop((*self.items[next_index].data.get()).replace(data));
        }

        self.index.store(next_index, Ordering::Release);

        self.writing.store(false, Ordering::Release);
    }

    fn index(&self) -> usize {
        self.index.load(Ordering::Acquire) & Self::LEN_MASK
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicU8};

    use super::*;

    #[derive(Clone)]
    struct Data<T>(T, Arc<AtomicU8>);

    impl<T> Drop for Data<T> {
        fn drop(&mut self) {
            self.1.fetch_add(1, Ordering::Release);
        }
    }

    #[test]
    fn test_cache() {
        let drop_count = Arc::new(AtomicU8::new(0));

        // Create initial data with drop counter
        let data = Data(String::from("first_value"), drop_count.clone());
        let cache: Cache<Data<String>> = Cache::new(data);

        // Test 1: Get data and verify it works
        let retrieved = cache.get_data();
        assert_eq!(retrieved.0, "first_value");

        // Drop the retrieved data - should increment drop count to 1
        drop(retrieved);
        assert_eq!(drop_count.load(Ordering::Acquire), 1);

        // Test 2: Get data again to ensure cache still works
        let retrieved2 = cache.get_data();
        assert_eq!(retrieved2.0, "first_value");
        drop(retrieved2);
        assert_eq!(drop_count.load(Ordering::Acquire), 2);

        // Test 3: Update the cache with new data
        let new_data = Data(String::from("second_value"), drop_count.clone());
        cache.update(new_data);

        // The update wrote to next_index (slot 1), replacing None (no drop count change)
        // Then it updated the index to point to slot 1
        // The old data in slot 0 is still there but no longer active
        assert_eq!(drop_count.load(Ordering::Acquire), 2);

        // Test 4: Get the updated data - should now return "second_value"
        let retrieved3 = cache.get_data();
        assert_eq!(retrieved3.0, "second_value");
        drop(retrieved3);
        assert_eq!(drop_count.load(Ordering::Acquire), 3);

        // Test 5: Update again - should find slot 2
        let third_data = Data(String::from("third_value"), drop_count.clone());
        cache.update(third_data);
        // Replaces None in slot 2, no drop count change
        assert_eq!(drop_count.load(Ordering::Acquire), 3);

        // Test 6: Get the latest data
        let retrieved4 = cache.get_data();
        assert_eq!(retrieved4.0, "third_value");
        drop(retrieved4);
        assert_eq!(drop_count.load(Ordering::Acquire), 4);

        // Test 7: Update again - should find slot 3
        let fourth_data = Data(String::from("fourth_value"), drop_count.clone());
        cache.update(fourth_data);
        assert_eq!(drop_count.load(Ordering::Acquire), 4);

        // Verify data is correct
        let retrieved5 = cache.get_data();
        assert_eq!(retrieved5.0, "fourth_value");
        drop(retrieved5);
        assert_eq!(drop_count.load(Ordering::Acquire), 5);

        // Test 8: Update again - should cycle back to slot 0 and replace "first_value"
        let fifth_data = Data(String::from("fifth_value"), drop_count.clone());
        cache.update(fifth_data);
        // This replaces "first_value" in slot 0, so drop count increments to 6
        assert_eq!(drop_count.load(Ordering::Acquire), 6);

        // Verify the new data is readable
        let final_retrieved = cache.get_data();
        assert_eq!(final_retrieved.0, "fifth_value");
        drop(final_retrieved);
        assert_eq!(drop_count.load(Ordering::Acquire), 7);

        // When cache is dropped, all slots with data are dropped:
        // slot 0: "fifth_value" (current) -> 8
        // slot 1: "second_value" (old) -> 9
        // slot 2: "third_value" (old) -> 10
        // slot 3: "fourth_value" (old) -> 11
        drop(cache);
        assert_eq!(drop_count.load(Ordering::Acquire), 11);
    }
}
