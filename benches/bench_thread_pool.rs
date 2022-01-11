use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use kvs::{KvStore, Result, KvsEngine, SledKvsEngine, KvsServer, KvsClient};
use kvs::thread_pool::*;
use tempfile::TempDir;
use rand::prelude::*;
use std::env::current_dir;
use num_cpus;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;
use predicates::str::{contains, is_empty};
use std::sync::mpsc;
use std::thread;
use crossbeam_utils::sync::WaitGroup;
use std::sync::Arc;
use std::time::Duration;


pub fn write(c: &mut Criterion) {
    let mut group = c.benchmark_group("heavy_write");
    let cpus = num_cpus::get() as u8;

    for i in &vec![1, 2, cpus] {
        
        group.bench_with_input(format!("shared_queued_{}", i), i, |b, i| {
            let engine = KvStore::open(&current_dir().unwrap()).unwrap();
            let pool = SharedRayonThreadPool::new(*i).unwrap();
            let server = KvsServer::new(engine, pool);
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
            //
            let server_handle = thread::spawn(move || {
                server.run(addr);
            });
            //
            let mut small_rng = SmallRng::from_seed([0; 16]);
            //

            b.iter(|| {
                let wg = WaitGroup::new();
                for _j in 1..10 {
                    let wg_clone = wg.clone();
                    let mut rng = small_rng.clone();
                    thread::spawn(move || {
                        let mut client = KvsClient::connect(addr).unwrap();
                        client.set(format!("key{}", rng.gen_range(1, 1 << 10)), "value".to_owned()).unwrap();
                        drop(wg_clone);
                    });
                }
                wg.wait();
            });
        });
    }

    group.finish();
}



pub fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("heavy_read");
    let cpus = num_cpus::get() as u8;

    for i in &vec![1, 2, cpus] {
        
        group.bench_with_input(format!("read_shared_queued_{}", i), i, |b, i| {
            let mut engine = KvStore::open(&current_dir().unwrap()).unwrap();
            for j in 1..1<<10 {
                engine.set(format!("key{}", j), "value".to_owned()).unwrap();
            }
            let pool = SharedRayonThreadPool::new(*i).unwrap();
            let server = KvsServer::new(engine, pool);
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4001);
            //
            thread::spawn(move || {
                server.run(addr);
            });
            //
            let mut small_rng = SmallRng::from_seed([0; 16]);
            b.iter(|| {
                let wg = WaitGroup::new();
                for _j in 1..100 {
                    let wg_clone = wg.clone();
                    let mut rng = small_rng.clone();
                    thread::spawn(move || {
                        let mut client = KvsClient::connect(addr).unwrap();
                        let value = client.get(format!("key{}", rng.gen_range(1, 1 << 10))).unwrap();
                        assert_eq!(value, Some("value".to_owned()));
                        drop(wg_clone);
                    });
                }
                wg.wait();
            });
        });
    }

    group.finish();
}



criterion_group!(benches, read);
criterion_main!(benches);