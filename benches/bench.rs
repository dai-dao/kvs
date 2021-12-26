use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use kvs::{KvStore, Result, KvsEngine, SledKvsEngine};
use tempfile::TempDir;
use rand::prelude::*;


pub fn write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");

    for i in &vec![8, 12] {
        
        group.bench_with_input(format!("kvs_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            let mut small_rng = SmallRng::from_seed([0; 16]);
            b.iter(|| store.set(format!("key{}", small_rng.gen_range(1, 1 << i)), "value".to_owned()).unwrap());
        });

        group.bench_with_input(format!("sled_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = SledKvsEngine::new(sled::open(TempDir::new().unwrap().path()).unwrap());
            let mut small_rng = SmallRng::from_seed([0; 16]);
            b.iter(|| store.set(format!("key{}", small_rng.gen_range(1, 1 << i)), "value".to_owned()).unwrap());
        });
    }

    group.finish();
}


// pub fn write(c: &mut Criterion) {
//     group.bench_function("kvs write", move |b| {
//         // Set up input, create a new kv store in a different directory for every iteration
//         b.iter_batched(|| {
//             let temp_dir = TempDir::new().unwrap();
//             KvStore::open(temp_dir.path()).unwrap()
//         }
//         , 
//         |mut store| write_engine(&mut store), 
//         BatchSize::SmallInput)
//     });

//     group.bench_function("sled write", move |b| {
//         b.iter_batched(|| 
//             SledKvsEngine::new(sled::open(TempDir::new().unwrap().path()).unwrap())
//         , 
//         |mut store| write_engine(&mut store), 
//         BatchSize::SmallInput)
//     });

//     group.finish();
// }


pub fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");

    for i in &vec![8, 12] {
        
        group.bench_with_input(format!("kvs_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            for j in 1..1 << i {
                store.set(format!("key{}", j), "value".to_owned()).unwrap();
            }
            let mut small_rng = SmallRng::from_seed([0; 16]);
            b.iter(|| store.get(format!("key{}", small_rng.gen_range(1, 1 << i))).unwrap());
        });

        group.bench_with_input(format!("sled_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = SledKvsEngine::new(sled::open(TempDir::new().unwrap().path()).unwrap());
            for j in 1..1 << i {
                store.set(format!("key{}", j), "value".to_owned()).unwrap();
            }
            let mut small_rng = SmallRng::from_seed([0; 16]);
            b.iter(|| store.get(format!("key{}", small_rng.gen_range(1, 1 << i))).unwrap());
        });
    }

    group.finish();
}




criterion_group!(benches, write, read);
criterion_main!(benches);