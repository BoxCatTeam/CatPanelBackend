use std::env::temp_dir;
use std::fs::{remove_dir_all, remove_file};

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rand::{Rng, thread_rng};

use cat_panel_backend::kv::{KVStore, LmdbStore, PersyStore, RedbStore};

fn bench_persy(c: &mut Criterion) {
    let path = temp_dir().join("cp_bench_persy");
    let mut store = PersyStore::new(&path).unwrap();
    let mut rng = thread_rng();

    let mut data = Vec::with_capacity(128);

    for _ in 0..128 {
        let i = rng.gen::<isize>().to_string();
        store.insert(&i, i.as_bytes().repeat(1000)).unwrap();
        data.push(i);
    }

    c.bench_function("kv-persy-read", |b| b.iter(|| {
        let _ = black_box(store.get(black_box(data.get(rng.gen_range(0..128)).unwrap())));
    }));
    remove_file(path.with_extension("persy")).unwrap();
}

fn bench_lmdb(c: &mut Criterion) {
    let path = temp_dir().join("cp_bench_lmdb");
    let mut store = LmdbStore::new(&path).unwrap();
    let mut rng = thread_rng();

    let mut data = Vec::with_capacity(128);

    for _ in 0..128 {
        let i = rng.gen::<isize>().to_string();
        store.insert(&i, i.as_bytes().repeat(1000)).unwrap();
        data.push(i);
    }

    c.bench_function("kv-lmdb-read", |b| b.iter(|| {
        let _ = black_box(store.get(black_box(data.get(rng.gen_range(0..128)).unwrap())));
    }));
    //drop(store);
    //remove_dir_all(path.with_extension("mdb")).unwrap();
}

fn bench_redb(c: &mut Criterion) {
    let path = temp_dir().join("cp_bench_redb");
    let mut store = RedbStore::new(&path).unwrap();
    let mut rng = thread_rng();

    let mut data = Vec::with_capacity(128);

    for _ in 0..128 {
        let i = rng.gen::<isize>().to_string();
        store.insert(&i, i.as_bytes().repeat(1000)).unwrap();
        data.push(i);
    }

    c.bench_function("kv-redb-read", |b| b.iter(|| {
        let _ = black_box(store.get(black_box(data.get(rng.gen_range(0..128)).unwrap())));
    }));
    remove_file(path.with_extension("redb")).unwrap();
}

criterion_group!(benches, bench_persy, bench_lmdb, bench_redb);
criterion_main!(benches);
