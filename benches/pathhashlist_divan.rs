use dirhash_rs::pathhash::pathhashspy::PathHashSpy;
use dirhash_rs::pathhash::PathHashProvider;
use dirhash_rs::pathhashlist::PathHashList;
use std::io::Write as _;
use std::path::Path;

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn create_large_spy_vec(count: usize) -> Vec<PathHashSpy> {
    let mut spies = Vec::with_capacity(count);

    for i in 0..count {
        let path_num = format!("/{}", i);
        let hash = format!("{:064?}", i);
        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(hash, &mut hash_bytes).unwrap();

        let spy = PathHashSpy::new(Path::new(&path_num).to_owned(), Some(hash_bytes), None);
        spies.push(spy);
    }

    spies
}

#[allow(dead_code)]
fn write_spy_vec_to_file(spies: &Vec<PathHashSpy>) -> std::io::Result<()> {
    let mut file = std::fs::File::create("spies.txt")?;

    for spy in spies {
        writeln!(
            &mut file,
            "{}  {}",
            hex::encode(spy.hash().unwrap()),
            spy.path().to_string_lossy()
        )
        .expect("Error while writing spies to file with writeln!");
    }

    Ok(())
}

mod compute_hash {
    use super::*;

    #[divan::bench]
    pub fn compute_hash(bencher: divan::Bencher) {
        let spies = create_large_spy_vec(1000);

        // Use this function to create a file containing all the hashes and paths of the spies. You can
        // then compute the overall hash with "sha256sum spies.txt" and place the expected overall hash
        // in the `assert_eq!()` down below.
        // write_spy_vec_to_file(&spies).expect("Error while writing spies vec");

        let mut pathhashlist = PathHashList::new(spies, None).expect("Can't create PathHashList");

        bencher.bench_local(|| pathhashlist.compute_hash());

        assert_eq!(pathhashlist.hash().unwrap(), b"\x1b\x80\xeb\xca\x22\x1d\xc9\xc8\x6e\xc4\x73\x30\x01\x33\xf9\x17\xfb\x01\xe9\x9d\xbc\xa8\xcb\xae\xe6\x2e\xce\x1d\x54\x96\xbf\xf2");
    }
}

fn main() {
    // Run registered benchmarks.
    divan::main();
}
