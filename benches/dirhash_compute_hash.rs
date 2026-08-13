use dirhash_rs::dirhash::DirHash;
use dirhash_rs::pathhash::pathhashspy::PathHashSpy;
use std::path::Path;

#[path = "../tests/common/mod.rs"]
mod common;

// use divan::AllocProfiler;
//
// #[global_allocator]
// static ALLOC: AllocProfiler = AllocProfiler::system();

mod compute_hash_with_spies {
    use super::*;

    fn create_large_spy_vec(count: usize) -> Vec<PathHashSpy> {
        let mut spies = Vec::with_capacity(count);

        for i in 0..count {
            let path_num = format!("/{}", i);
            let hash = format!("{:064?}", i);
            let mut hash_bytes = [0u8; 32];
            hex::decode_to_slice(hash, &mut hash_bytes).unwrap();

            let spy = PathHashSpy::new(Path::new(&path_num).to_owned(), None, Some(hash_bytes));
            spies.push(spy);
        }

        spies
    }

    #[divan::bench(args = [100, 1000, 10000, 100000], max_time = 5)]
    pub fn compute_hash_serial(bencher: divan::Bencher, file_count: usize) {
        bencher
            .with_inputs(|| {
                let spies = create_large_spy_vec(file_count);
                DirHash::new().with_files(spies)
            })
            .bench_local_values(|mut dh| dh.compute_hash_serial());
    }

    #[divan::bench(args = [100, 1000, 10000, 100000], max_time = 5)]
    pub fn compute_hash_rayon1(bencher: divan::Bencher, file_count: usize) {
        bencher
            .with_inputs(|| {
                let spies = create_large_spy_vec(file_count);
                DirHash::new().with_files(spies)
            })
            .bench_local_values(|mut dh| dh.compute_hash_rayon1());
    }

    #[divan::bench(args = [100, 1000, 10000, 100000], max_time = 5)]
    pub fn compute_hash_rayon2(bencher: divan::Bencher, file_count: usize) {
        bencher
            .with_inputs(|| {
                let spies = create_large_spy_vec(file_count);
                DirHash::new().with_files(spies)
            })
            .bench_local_values(|mut dh| dh.compute_hash_rayon2());
    }
}

mod compute_hash_with_tempfiles {
    use super::*;

    #[divan::bench(args = [100, 1000, 10000], max_time = 5)]
    pub fn compute_hash_serial(bencher: divan::Bencher, file_count: usize) {
        // :TODO: extract and use for all benchmarks
        let dir = common::creating_tempdir(
            None,
            file_count,
            &["a", "b"],
            file_count,
            &["c", "d"],
            file_count,
            true,
        );

        bencher
            .with_inputs(|| {
                DirHash::new()
                    .with_files_from_dir(dir.path(), true, false, false, false)
                    .expect("Can't create DirHash")
            })
            .bench_local_values(|mut dh| dh.compute_hash_serial());

        dir.close().expect("Can't close tempdir");
    }

    #[divan::bench(args = [100, 1000, 10000], max_time = 5)]
    pub fn compute_hash_rayon1(bencher: divan::Bencher, file_count: usize) {
        let dir = common::creating_tempdir(
            None,
            file_count,
            &["a", "b"],
            file_count,
            &["c", "d"],
            file_count,
            true,
        );

        bencher
            .with_inputs(|| {
                DirHash::new()
                    .with_files_from_dir(dir.path(), true, false, false, false)
                    .expect("Can't create DirHash")
            })
            .bench_local_values(|mut dh| dh.compute_hash_rayon1());

        dir.close().expect("Can't close tempdir");
    }

    #[divan::bench(args = [100, 1000, 10000], max_time = 5)]
    pub fn compute_hash_rayon2(bencher: divan::Bencher, file_count: usize) {
        let dir = common::creating_tempdir(
            None,
            file_count,
            &["a", "b"],
            file_count,
            &["c", "d"],
            file_count,
            true,
        );

        bencher
            .with_inputs(|| {
                DirHash::new()
                    .with_files_from_dir(dir.path(), true, false, false, false)
                    .expect("Can't create DirHash")
            })
            .bench_local_values(|mut dh| dh.compute_hash_rayon2());

        dir.close().expect("Can't close tempdir");
    }
}

fn main() {
    // Run registered benchmarks.
    divan::main();
}
