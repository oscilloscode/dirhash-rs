use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dirhash_rs::pathhash::pathhashspy::PathHashSpy;
use dirhash_rs::pathhash::PathHashProvider;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn parse_benchmark(c: &mut Criterion) {
    let spies = vec![
            PathHashSpy::new(
                Path::new("/some/path").to_owned(),
                None,
                Some(*b"\xd8\x3b\xa8\x04\x20\xec\x99\xbc\xb1\x43\xdf\x16\xa0\x0c\x39\xa5\x6c\x14\x03\x41\xe4\x44\x6a\xe9\xb5\xe8\xb5\xa6\xd1\x81\x16\xed"), // hash of "/some/path"
            ),
            PathHashSpy::new(
                Path::new("/other/path").to_owned(),
                Some(*b"\x59\xea\xd6\x2a\x5f\x16\xe4\xee\x2f\x7d\xe8\x9e\x52\xf9\x78\xd6\xf1\x5e\x97\xf3\x87\x25\x5d\xd7\x7e\xd3\xc7\x2f\x88\x88\x28\x55"), // hash of "/other/path"
                None,
            ),
        ];

    println!("{:?}", spies[1].hash());
    c.bench_function("sleep 10ms", |b| {
        b.iter(|| thread::sleep(Duration::from_millis(10)))
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
