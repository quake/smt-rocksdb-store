use rocksdb::{prelude::Open, OptimisticTransactionDB, DB};
use sparse_merkle_tree::{blake2b::Blake2bHasher, SparseMerkleTree, H256};

use crate::default_store::DefaultStore;

use super::{new_blake2b, MemoryStoreSMT, Word};

type DefaultStoreSMT<'a, T, W> = SparseMerkleTree<Blake2bHasher, Word, DefaultStore<'a, T, W>>;

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
        let mut memory_store_smt = MemoryStoreSMT::new(H256::default(), Default::default());
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
        let db = DB::open_default(tmp_dir.path()).unwrap();
        let rocksdb_store = DefaultStore::new(&db);
        let mut rocksdb_store_smt = DefaultStoreSMT::new(H256::default(), rocksdb_store);
        for (key, value) in kvs.iter() {
            rocksdb_store_smt
                .update(key.clone(), value.clone())
                .unwrap();
        }
        let root = rocksdb_store_smt.root().clone();
        let snapshot = db.snapshot();
        let rocksdb_store_smt =
            DefaultStoreSMT::new(root.clone(), DefaultStore::<_, ()>::new(&snapshot));
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
        let db = OptimisticTransactionDB::open_default(tmp_dir.path()).unwrap();
        let tx = db.transaction_default();
        let rocksdb_store = DefaultStore::new(&tx);
        let mut rocksdb_store_smt = DefaultStoreSMT::new(H256::default(), rocksdb_store);
        for (key, value) in kvs.iter() {
            rocksdb_store_smt
                .update(key.clone(), value.clone())
                .unwrap();
        }
        tx.commit().unwrap();

        let root = rocksdb_store_smt.root().clone();
        let snapshot = db.snapshot();
        let rocksdb_store_smt =
            DefaultStoreSMT::new(root.clone(), DefaultStore::<_, ()>::new(&snapshot));
        let proof = rocksdb_store_smt
            .merkle_proof(vec![kvs[0].0.clone()])
            .unwrap();
        (root, proof)
    };
    assert_eq!(root1, root3);
    assert_eq!(proof1, proof3);
}
