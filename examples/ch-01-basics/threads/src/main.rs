use std::thread;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);

    println!("Hello from the main thread");

    t1.join().unwrap();
    t2.join().unwrap();



    let numbers = vec![1, 2, 3];

    thread::spawn(move || {
        for n in &numbers {
            println!("{n}");
        }
    }).join().unwrap();



    let numbers = Vec::from_iter(0..=1000);

    let t = thread::spawn(move || {
        let len = numbers.len();
        let sum = numbers.iter().sum::<usize>();
        sum / len
    });

    let average = t.join().unwrap();

    println!("average: {average}");



    // Scoped threads

    let numbers = vec![1, 2, 3];

    thread::scope(|s| {
        s.spawn(|| {
            println!("length: {}", numbers.len());
        });
        s.spawn(|| {
            for n in &numbers {
                println!("{n}");
            }
        });
    });



    // Shared Ownership and Reference Counting

    // Statics

    static X: [i32; 3] = [1, 2, 3];

    let t1 = thread::spawn(|| dbg!(&X));
    let t2 = thread::spawn(|| dbg!(&X));
    t1.join().unwrap();
    t2.join().unwrap();

    // Leaking

    let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3]));

    let t1 = thread::spawn(move || dbg!(x));
    let t2 = thread::spawn(move || dbg!(x));
    t1.join().unwrap();
    t2.join().unwrap();

    // Reference Counting

    let a = Rc::new([1, 2, 3]);
    let b = a.clone();

    assert_eq!(a.as_ptr(), b.as_ptr()); // Same allocation!


    let a = Arc::new([1, 2, 3]);
    let b = a.clone();

    let t1 = thread::spawn(move || dbg!(a));
    let t2 = thread::spawn(move || dbg!(b));
    t1.join().unwrap();
    t2.join().unwrap();
}

fn f() {
    println!("Hello from another thread!");

    let id = thread::current().id();
    println!("This is my thread id: {id:?}");
}
