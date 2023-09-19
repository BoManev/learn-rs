use std::cell::Cell;
use std::cell::RefCell;

pub struct SArena<T> {
    chunks: RefCell<Vec<Vec<T>>>,
}

impl<T> SArena<T> {
    pub fn new() -> SArena<T> {
        SArena {
            chunks: RefCell::new(vec![Vec::with_capacity(10)]),
        }
    }

    pub fn allocate(&self, value: T) -> &T {
        let mut chunks = self.chunks.borrow_mut();
        if chunks.last().unwrap().len() >= chunks.last().unwrap().capacity() {
            let new_cap = chunks.last().unwrap().capacity() * 2;
            chunks.push(Vec::with_capacity(new_cap))
        }

        chunks.last_mut().unwrap().push(value);
        let val_ptr: *const T = chunks.last().unwrap().last().unwrap();
        unsafe {
            // SAFETY: we allocate a new vec when capacity is reached, instead of growing and rellocating the previous one
            // def raw pointer to "extend" lifetime
            &*val_ptr
        }
    }
}

struct Node<'arena> {
    value: u32,
    next: Cell<Option<&'arena Node<'arena>>>,
}

// impl<'arena> Drop for Node<'arena> { fn drop(&mut self) {} }

pub fn sarena() {
    let arena = SArena::new();
    let c = arena.allocate(Node {
        value: 2,
        next: Cell::new(None),
    });
    let b = arena.allocate(Node {
        value: 1,
        next: Cell::new(Some(c)),
    });
    let a = arena.allocate(Node {
        value: 0,
        next: Cell::new(Some(b)),
    });
    c.next.set(Some(b));

    let mut node = a;
    let mut vals = Vec::new();
    for _ in 0..5 {
        vals.push(node.value);
        node = node.next.get().unwrap()
    }
    assert_eq!(vals, [0, 1, 2, 1, 2])
}
