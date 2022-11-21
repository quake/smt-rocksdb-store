use rocksdb::{
    prelude::{GetColumnFamilys, OpenCF},
    OptimisticTransactionDB, Options, DB,
};
use sparse_merkle_tree::{blake2b::Blake2bHasher, SparseMerkleTree, H256};

use crate::cf_store::{ColumnFamilyStore, ColumnFamilyStoreMultiTree};

use super::{new_blake2b, MemoryStoreSMT, Word};

type ColumnFamilyStoreSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, Word, ColumnFamilyStore<'a, T, W>>;

#[test]
fn test_store_functions() {
    let kvs = "The quick brown fox jumps over the lazy dog"
        .split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let mut buf = [0u8; 32];
            let mut hasher = new_blake2b();
            hasher.update(&(i as u32).to_le_bytes());
            hasher.finalize(&mut buf);
            (buf.into(), Word(word.to_string()))
        })
        .collect::<Vec<(H256, Word)>>();

    // generate a merkle tree with a memory store
    let (root1, proof1) = {
        let mut memory_store_smt = MemoryStoreSMT::new_with_store(Default::default()).unwrap();
        for (key, value) in kvs.iter() {
            memory_store_smt.update(key.clone(), value.clone()).unwrap();
        }
        let root = memory_store_smt.root().clone();
        let proof = memory_store_smt
            .merkle_proof(vec![kvs[0].0.clone()])
            .unwrap();
        (root, proof)
    };

    // generate a merkle tree with a rocksdb store
    let (root2, proof2) = {
        let tmp_dir = tempfile::Builder::new().tempdir().unwrap();
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        let db = DB::open_cf(&options, tmp_dir.path(), vec!["cf1", "cf2"]).unwrap();
        let branch_col = db.cf_handle("cf1").unwrap();
        let leaf_col = db.cf_handle("cf2").unwrap();
        let rocksdb_store = ColumnFamilyStore::new(&db, branch_col, leaf_col);
        let mut rocksdb_store_smt = ColumnFamilyStoreSMT::new_with_store(rocksdb_store).unwrap();
        for (key, value) in kvs.iter() {
            rocksdb_store_smt
                .update(key.clone(), value.clone())
                .unwrap();
        }
        let root = rocksdb_store_smt.root().clone();
        let snapshot = db.snapshot();
        let rocksdb_store_smt = ColumnFamilyStoreSMT::new(
            root.clone(),
            ColumnFamilyStore::<_, ()>::new(&snapshot, branch_col, leaf_col),
        );
        let proof = rocksdb_store_smt
            .merkle_proof(vec![kvs[0].0.clone()])
            .unwrap();
        (root, proof)
    };
    assert_eq!(root1, root2);
    assert_eq!(proof1, proof2);

    // generate a merkle tree with a rocksdb store in a transaction
    let (root3, proof3) = {
        let tmp_dir = tempfile::Builder::new().tempdir().unwrap();
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        let db =
            OptimisticTransactionDB::open_cf(&options, tmp_dir.path(), vec!["cf1", "cf2"]).unwrap();
        let branch_col = db.cf_handle("cf1").unwrap();
        let leaf_col = db.cf_handle("cf2").unwrap();
        let tx = db.transaction_default();
        let rocksdb_store = ColumnFamilyStore::new(&tx, branch_col, leaf_col);
        let mut rocksdb_store_smt = ColumnFamilyStoreSMT::new_with_store(rocksdb_store).unwrap();
        for (key, value) in kvs.iter() {
            rocksdb_store_smt
                .update(key.clone(), value.clone())
                .unwrap();
        }
        tx.commit().unwrap();

        let root = rocksdb_store_smt.root().clone();
        let snapshot = db.snapshot();
        let rocksdb_store_smt = ColumnFamilyStoreSMT::new(
            root.clone(),
            ColumnFamilyStore::<_, ()>::new(&snapshot, branch_col, leaf_col),
        );
        let proof = rocksdb_store_smt
            .merkle_proof(vec![kvs[0].0.clone()])
            .unwrap();
        (root, proof)
    };
    assert_eq!(root1, root3);
    assert_eq!(proof1, proof3);
}

type ColumnFamilyStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, Word, ColumnFamilyStoreMultiTree<'a, T, W>>;

#[test]
fn test_multi_trees_store_functions() {
    let kvs = "The quick brown fox jumps over the lazy dog"
        .split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let mut buf = [0u8; 32];
            let mut hasher = new_blake2b();
            hasher.update(&(i as u32).to_le_bytes());
            hasher.finalize(&mut buf);
            (buf.into(), Word(word.to_string()))
        })
        .collect::<Vec<(H256, Word)>>();

    // generate a merkle tree with a memory store
    let (root1, proof1) = {
        let mut memory_store_smt = MemoryStoreSMT::new_with_store(Default::default()).unwrap();
        for (key, value) in kvs.iter() {
            memory_store_smt.update(key.clone(), value.clone()).unwrap();
        }
        let root = memory_store_smt.root().clone();
        let proof = memory_store_smt
            .merkle_proof(vec![kvs[0].0.clone()])
            .unwrap();
        (root, proof)
    };

    // generate a merkle tree with a rocksdb store
    {
        let tmp_dir = tempfile::Builder::new().tempdir().unwrap();
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        let db = DB::open_cf(&options, tmp_dir.path(), vec!["cf1", "cf2"]).unwrap();
        let branch_col = db.cf_handle("cf1").unwrap();
        let leaf_col = db.cf_handle("cf2").unwrap();

        let rocksdb_store1 = ColumnFamilyStoreMultiTree::new(b"tree1", &db, &branch_col, &leaf_col);
        let rocksdb_store2 = ColumnFamilyStoreMultiTree::new(b"tree2", &db, &branch_col, &leaf_col);
        let mut smt1 = ColumnFamilyStoreMultiSMT::new_with_store(rocksdb_store1).unwrap();
        let mut smt2 = ColumnFamilyStoreMultiSMT::new_with_store(rocksdb_store2).unwrap();
        for (key, value) in kvs.iter() {
            smt1.update(key.clone(), value.clone()).unwrap();
            smt2.update(key.clone(), value.clone()).unwrap();
        }
        smt2.update(kvs.first().unwrap().0.clone(), Word::default())
            .unwrap();

        let root_tree1 = smt1.root().clone();
        let root_tree2 = smt2.root().clone();
        let snapshot = db.snapshot();
        let smt1 = ColumnFamilyStoreMultiSMT::new(
            root1.clone(),
            ColumnFamilyStoreMultiTree::<_, ()>::new(b"tree1", &snapshot, &branch_col, &leaf_col),
        );
        let proof_tree1 = smt1.merkle_proof(vec![kvs[0].0.clone()]).unwrap();

        assert_eq!(root1, root_tree1);
        assert_eq!(proof1, proof_tree1);
        assert_ne!(root_tree1, root_tree2);
    };
}
