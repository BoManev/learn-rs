use std::cell::Cell;
#[derive(Debug)]
struct Node<'a> {
    value: u32,
    next: Cell<Option<&'a Node<'a>>>,
}

pub fn naive() {
    // Satisfy cycle lifetime dependcy by using a composite value
    let (a, b, c) = (
        Node {
            value: 0,
            next: Cell::new(None),
        },
        Node {
            value: 1,
            next: Cell::new(None),
        },
        Node {
            value: 2,
            next: Cell::new(None),
        },
    );

    a.next.set(Some(&b));
    b.next.set(Some(&c));
    c.next.set(Some(&b));

    let mut node = &a;
    let mut vals = Vec::new();
    for _ in 0..5 {
        vals.push(node.value);
        node = node.next.get().unwrap()
    }
    assert_eq!(vals, [0, 1, 2, 1, 2])
}
