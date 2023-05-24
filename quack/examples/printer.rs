use std::fs::File;
use std::io::Write;
use bincode;
use quack::{Quack, PowerSumQuack};

fn main() {
    let mut q = PowerSumQuack::<u32>::new(20);
    for id in [1, 2, 3] {
        q.insert(id);
    }
    let bytes = bincode::serialize(&q).unwrap();
    let mut f = File::create("quack.out").unwrap();
    f.write_all(&bytes).unwrap();
}
