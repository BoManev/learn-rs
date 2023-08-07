use crate::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

struct RcInner<T> {
    value: T,
    refs: Cell<usize>,
}
pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Rc<T> {
    pub fn new(v: T) -> Self {
        let inner = Box::new(RcInner {
            value: v,
            refs: Cell::new(1),
        });
        Rc {
            // deref the box won't work since it will be droped (dealloc) after new returns
            // SAFETY: Box does not give us a null pointer
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}
impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let refs = inner.refs.get();
        inner.refs.set(refs + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a Box that is only deallocaed when the last RC goes away
        // we have an Rc, therefore the Box has not been deallocated, so deref is fine
        &unsafe { self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let refs = inner.refs.get();
        if refs == 1 {
            drop(inner);
            // SAFETY: we are the _only_ Rc left, and we are being dropped.
            // therefore, after us, there will be no Rc's, and no references to T
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            // There are other Rcs, so don't drop the Box
            inner.refs.set(refs - 1);
        }
    }
}

