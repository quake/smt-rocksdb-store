use criterion::{criterion_group, BenchmarkId, Criterion};
use rand::{seq::IteratorRandom, thread_rng};
use rocksdb::{
    prelude::{GetColumnFamilys, OpenCF},
    OptimisticTransactionDB, Options,
};

use smt_rocksdb_store::cf_store::{ColumnFamilyStore, ColumnFamilyStoreMultiTree};
use sparse_merkle_tree::{blake2b::Blake2bHasher, SparseMerkleTree};
use tempfile::{Builder, TempDir};

use super::{random_kvs, V};

type ColumnFamilyStoreSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, V, ColumnFamilyStore<'a, T, W>>;

type ColumnFamilyStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, V, ColumnFamilyStoreMultiTree<'a, T, W>>;

// return temp dir also to make sure it's not dropped automatically
fn open_db() -> (OptimisticTransactionDB, TempDir) {
    let tmp_dir = Builder::new().tempdir().unwrap();
    let mut options = Options::default();
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    (
        OptimisticTransactionDB::open_cf(&options, tmp_dir.path(), vec!["cf1", "cf2"]).unwrap(),
        tmp_dir,
    )
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cf_smt_update");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = ColumnFamilyStore::new(&tx, branch_col, leaf_col);
                let mut rocksdb_store_smt =
                    ColumnFamilyStoreSMT::new_with_store(rocksdb_store).unwrap();
                for (key, value) in random_kvs(count) {
                    rocksdb_store_smt.update(key, value).unwrap();
                }
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cf_smt_update_all");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store = ColumnFamilyStore::new(&tx, branch_col, leaf_col);
                let mut rocksdb_store_smt =
                    ColumnFamilyStoreSMT::new_with_store(rocksdb_store).unwrap();
                rocksdb_store_smt.update_all(random_kvs(count)).unwrap();
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cf_smt_generate_proof");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            let tx = db.transaction_default();
            let rocksdb_store = ColumnFamilyStore::new(&tx, branch_col, leaf_col);
            let mut rocksdb_store_smt =
                ColumnFamilyStoreSMT::new_with_store(rocksdb_store).unwrap();
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
                let rocksdb_store: ColumnFamilyStore<_, ()> =
                    ColumnFamilyStore::new(&snapshot, branch_col, leaf_col);
                let rocksdb_store_smt = ColumnFamilyStoreSMT::new(root, rocksdb_store);
                rocksdb_store_smt.merkle_proof(keys).unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cf_smt_multi_tree_update");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store =
                    ColumnFamilyStoreMultiTree::new(b"tree1", &tx, branch_col, leaf_col);
                let mut rocksdb_store_smt =
                    ColumnFamilyStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
                for (key, value) in random_kvs(count) {
                    rocksdb_store_smt.update(key, value).unwrap();
                }
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cf_smt_multi_tree_update_all");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            b.iter(|| {
                let tx = db.transaction_default();
                let rocksdb_store =
                    ColumnFamilyStoreMultiTree::new(b"tree1", &tx, branch_col, leaf_col);
                let mut rocksdb_store_smt =
                    ColumnFamilyStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
                rocksdb_store_smt.update_all(random_kvs(count)).unwrap();
                tx.commit().unwrap();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cf_smt_multi_tree_generate_proof");
    for count in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let (db, _tmp_dir) = open_db();
            let branch_col = db.cf_handle("cf1").unwrap();
            let leaf_col = db.cf_handle("cf2").unwrap();
            let tx = db.transaction_default();
            let rocksdb_store =
                ColumnFamilyStoreMultiTree::new(b"tree1", &tx, branch_col, leaf_col);
            let mut rocksdb_store_smt =
                ColumnFamilyStoreMultiSMT::new_with_store(rocksdb_store).unwrap();
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
                let rocksdb_store: ColumnFamilyStoreMultiTree<_, ()> =
                    ColumnFamilyStoreMultiTree::new(b"tree1", &snapshot, branch_col, leaf_col);
                let rocksdb_store_smt = ColumnFamilyStoreMultiSMT::new(root, rocksdb_store);
                rocksdb_store_smt.merkle_proof(keys).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark);
