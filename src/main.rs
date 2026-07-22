mod arena;

use arena::Arena;

// next thing to do:
//     - implement freelists

fn main() {
    println!("Hello, world!");
    let mut arena = Arena::new(1024);
    let x = arena.alloc(10);
    arena.alloc(42);
    arena.alloc(84);
    arena.alloc(123);

    arena.free(x).expect("failed to free");
    // arena.free(x).expect("failed to free"); // double free

    for (idx, chunk) in arena.chunks().enumerate() {
        println!(
            "chunk #{} -> {{ (size: {:#x}, state: {:?}) }}",
            idx,
            chunk.allocated_size,
            chunk.state
        )
    }
}
