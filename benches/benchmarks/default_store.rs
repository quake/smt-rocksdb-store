use criterion::{criterion_group, BenchmarkId, Criterion};
use rand::{seq::IteratorRandom, thread_rng};
use rocksdb::{prelude::Open, OptimisticTransactionDB};

use smt_rocksdb_store::default_store::{DefaultStore, DefaultStoreMultiTree};
use sparse_merkle_tree::{blake2b::Blake2bHasher, SparseMerkleTree};
use tempfile::{Builder, TempDir};

use super::{random_kvs, V};

type DefaultStoreSMT<'a, T, W> = SparseMerkleTree<Blake2bHasher, V, DefaultStore<'a, T, W>>;
type DefaultStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, V, DefaultStoreMultiTree<'a, T, W>>;

// return temp dir also to make sure it's not dropped automatically
fn open_db() -> (OptimisticTransactionDB, TempDir) {
    let tmp_dir = Builder::new().tempdir().unwrap();
    (
        OptimisticTransactionDB::open_default(tmp_dir.path()).unwrap(),
        tmp_dir,
    )
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("default_smt_update");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = DefaultStore::new(&tx);
                let mut rocksdb_store_smt = DefaultStoreSMT::new_with_store(rocksdb_store).unwrap();
                for (key, value) in random_kvs(count) {
                    rocksdb_store_smt.update(key, value).unwrap();
                }
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("default_smt_update_all");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = DefaultStore::new(&tx);
                let mut rocksdb_store_smt = DefaultStoreSMT::new_with_store(rocksdb_store).unwrap();
                rocksdb_store_smt.update_all(random_kvs(count)).unwrap();
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("default_smt_generate_proof");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let tx = db.transaction_default();
            let rocksdb_store = DefaultStore::new(&tx);
            let mut rocksdb_store_smt = DefaultStoreSMT::new_with_store(rocksdb_store).unwrap();
            let kvs = random_kvs(count);
            rocksdb_store_smt.update_all(kvs.clone()).unwrap();
            let root = rocksdb_store_smt.root().clone();
            tx.commit().unwrap();

            let mut rng = thread_rng();
            b.iter(|| {
                let keys = kvs
                    .iter()
                    .choose_multiple(&mut rng, count / 25)
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect();
                let snapshot = db.snapshot();
                let rocksdb_store: DefaultStore<_, ()> = DefaultStore::new(&snapshot);
                let rocksdb_store_smt = DefaultStoreSMT::new(root, rocksdb_store);
                rocksdb_store_smt.merkle_proof(keys).unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("default_multi_tree_smt_update");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = DefaultStoreMultiTree::new(b"tree1", &tx);
                let mut rocksdb_store_smt =
                    DefaultStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
                for (key, value) in random_kvs(count) {
                    rocksdb_store_smt.update(key, value).unwrap();
                }
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("default_multi_tree_smt_update_all");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = DefaultStoreMultiTree::new(b"tree1", &tx);
                let mut rocksdb_store_smt =
                    DefaultStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
                rocksdb_store_smt.update_all(random_kvs(count)).unwrap();
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("default_multi_tree_smt_generate_proof");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let tx = db.transaction_default();
            let rocksdb_store = DefaultStoreMultiTree::new(b"tree1", &tx);
            let mut rocksdb_store_smt =
                DefaultStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
            let kvs = random_kvs(count);
            rocksdb_store_smt.update_all(kvs.clone()).unwrap();
            let root = rocksdb_store_smt.root().clone();
            tx.commit().unwrap();

            let mut rng = thread_rng();
            b.iter(|| {
                let keys = kvs
                    .iter()
                    .choose_multiple(&mut rng, count / 25)
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect();
                let snapshot = db.snapshot();
                let rocksdb_store: DefaultStoreMultiTree<_, ()> =
                    DefaultStoreMultiTree::new(b"tree1", &snapshot);
                let rocksdb_store_smt = DefaultStoreMultiSMT::new(root, rocksdb_store);
                rocksdb_store_smt.merkle_proof(keys).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark);
