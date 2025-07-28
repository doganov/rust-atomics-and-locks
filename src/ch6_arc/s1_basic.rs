use std::ops::Deref;
use std::ptr::NonNull;

use crate::alt::sync::atomic::AtomicUsize;
use crate::alt::sync::atomic::Ordering::Acquire;
use crate::alt::sync::atomic::Ordering::Relaxed;
use crate::alt::sync::atomic::Ordering::Release;
use crate::alt::sync::atomic::fence;

struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                ref_count: AtomicUsize::new(1),
                data,
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        if self.data().ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Arc {
            ptr: self.ptr,
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

#[cfg(test)]
fn test() {
    use crate::alt::thread;


    // Loom's AtomicUsize::new() is not const, so we can't use it to initialize
    // static directly.  We can use Box::leak instead, but then it becomes
    // impossible to reclaim the memory leaked when running the test many times.
    // So instead, we manage num_drops's lifetime with Arc itself.
    let num_drops = Arc::new(AtomicUsize::new(0));

    // Since num_drops is not static, we can't refer to it in drop(), so we have
    // to pass it explicitly.
    struct DetectDrop {
        num_drops: Arc<AtomicUsize>,
    }

    impl DetectDrop {
        fn new(num_drops: Arc<AtomicUsize>) -> Self {
            Self { num_drops }
        }
    }

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            self.num_drops.fetch_add(1, Relaxed);
        }
    }

    // Create two Arcs sharing an object containing a string
    // and a DetectDrop, to detect when it's dropped.
    //let x = Arc::new(("hello", DetectDrop::new(&num_drops)));
    let x = Arc::new(("hello", DetectDrop::new(num_drops.clone())));
    let y = x.clone();

    // Send x to another thread, and use it there.
    let t = thread::spawn(move || {
        assert_eq!(x.0, "hello");
    });

    // In parallel, y should still be usable here.
    assert_eq!(y.0, "hello");

    // Wait for the thread to finish.
    t.join().unwrap();

    // One Arc, x, should be dropped by now.
    // We still have y, so the object shouldn't have been dropped yet.
    assert_eq!(num_drops.load(Relaxed), 0);

    // Drop the remaining `Arc`.
    drop(y);

    // Now that `y` is dropped too,
    // the object should've been dropped.
    assert_eq!(num_drops.load(Relaxed), 1);
}

#[cfg(not(loom))]
#[test]
fn test_not_loom() {
    test()
}

#[cfg(loom)]
#[test]
fn test_loom() {
    loom::model(|| test());
}
