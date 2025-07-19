use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;

static mut DATA: String = String::new();
static LOCKED: AtomicBool = AtomicBool::new(false);

#[allow(static_mut_refs)]
fn f() {
    if LOCKED.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
        // Safety: We hold the exclusive lock, so nothing else is accessing DATA.
        unsafe { DATA.push('!') };
        LOCKED.store(false, Release);
    }
}

#[allow(static_mut_refs)]
fn main() {
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(f);
        }
    });
    // DATA now contains at least one exclamation mark (and maybe more).
    println!("len: {}", unsafe { DATA.len() });
    assert!(unsafe { DATA.len() > 0 });
    assert!(unsafe { DATA.chars().all(|c| c == '!') });
}
