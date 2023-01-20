use std::fs::File;
use std::io::Write;
use bincode;
use quack::Quack;

fn main() {
    let mut q = Quack::new(20);
    for id in [1, 2, 3] {
        q.insert(id);
    }
    let bytes = bincode::serialize(&q).unwrap();
    let mut f = File::create("quack.out").unwrap();
    f.write_all(&bytes).unwrap();
}