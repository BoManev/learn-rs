use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

// implied by UnsafeCell
// impl<T> !Sync for Cell<T> {}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: we know no-one else is concurrently mutating self.value (bacause !Sync)
        // SAFETY: we know we're not invalidating any references, because we never give any out
        unsafe { *self.value.get() = value };
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.value.get() }
    }
}

#[cfg(test)]
mod race_threads {
    use super::Cell;

    unsafe impl<T> Sync for Cell<T> {}
    #[test]
    fn bad() {
        use std::sync::Arc;
        let x = Arc::new(Cell::new(0));
        let x1 = Arc::clone(&x);
        let t1 = std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                let x = x1.get();
                x1.set(x + 1);
            }
        });
        let x2 = Arc::clone(&x);
        let t2 = std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                let x = x2.get();
                x2.set(x + 1);
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        assert_ne!(x.get(), 2_000_000);
    }
}
