mod arena;

use arena::Arena;

// next thing to do:
//     - implement freelists

fn main() {
    println!("Hello, world!");
    let mut arena = Arena::new(1024);
    let x = arena.alloc(10);
    let y = arena.alloc(42);
    arena.alloc(84);
    arena.alloc(123);

    arena.free(x).expect("failed to free");
    arena.free(y).expect("failed to free");
    // arena.free(x).expect("failed to free"); // double free

    arena.dump();
}
