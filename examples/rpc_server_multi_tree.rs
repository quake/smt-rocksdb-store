use std::env;
use std::net::SocketAddr;

use jsonrpsee::core::{async_trait, Error};
use jsonrpsee::http_server::HttpServerBuilder;
use jsonrpsee::proc_macros::rpc;

use rocksdb::{
    prelude::{Iterate, Open},
    DBVector, OptimisticTransactionDB,
};
use rocksdb::{Direction, IteratorMode};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::blake2b::Blake2bHasher;
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::{SparseMerkleTree, H256};

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SmtKey(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SmtValue(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SmtRoot(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SmtProof(#[serde_as(as = "serde_with::hex::Hex")] Vec<u8>);

impl Value for SmtValue {
    fn to_h256(&self) -> H256 {
        self.0.into()
    }

    fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl From<DBVector> for SmtValue {
    fn from(vec: DBVector) -> Self {
        SmtValue(vec.as_ref().try_into().expect("stored value is 32 bytes"))
    }
}

impl AsRef<[u8]> for SmtValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

type DefaultStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, SmtValue, DefaultStoreMultiTree<'a, T, W>>;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "update_all")]
    async fn update_all(&self, tree: &str, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error>;

    #[method(name = "merkle_proof")]
    async fn merkle_proof(&self, tree: &str, keys: Vec<SmtKey>) -> Result<SmtProof, Error>;

    #[method(name = "clear")]
    async fn clear(&self, tree: &str) -> Result<(), Error>;
}

pub struct RpcServerImpl {
    db: OptimisticTransactionDB,
}

impl RpcServerImpl {
    fn new(db: OptimisticTransactionDB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn update_all(&self, tree: &str, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error> {
        let kvs: Vec<(H256, SmtValue)> = kvs.into_iter().map(|(k, v)| (k.0.into(), v)).collect();

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt =
            DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(tree.as_bytes(), &tx))
                .unwrap();
        rocksdb_store_smt.update_all(kvs).expect("update_all error");
        tx.commit().expect("db commit error");
        Ok(SmtRoot(rocksdb_store_smt.root().clone().into()))
    }

    async fn merkle_proof(&self, tree: &str, keys: Vec<SmtKey>) -> Result<SmtProof, Error> {
        let keys: Vec<H256> = keys.into_iter().map(|k| k.0.into()).collect();
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::<_, ()>::new(tree.as_bytes(), &snapshot),
        )
        .unwrap();
        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())
            .expect("merkle_proof error");
        Ok(SmtProof(proof.compile(keys).expect("compile error").0))
    }

    async fn clear(&self, tree: &str) -> Result<(), Error> {
        // OptimisticTransactionDB does not support delete_range, so we have to iterate all keys and update them to zero as a workaround
        let snapshot = self.db.snapshot();
        let prefix = tree.as_bytes();
        let prefix_len = prefix.len();
        let leaf_key_len = prefix_len + 32;
        let kvs: Vec<(H256, SmtValue)> = snapshot
            .iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(k, _)| {
                if k.len() != leaf_key_len {
                    None
                } else {
                    let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                    Some((leaf_key.into(), SmtValue::zero()))
                }
            })
            .collect();

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt =
            DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(tree.as_bytes(), &tx))
                .unwrap();
        rocksdb_store_smt.update_all(kvs).expect("update_all error");
        tx.commit().expect("db commit error");
        assert_eq!(rocksdb_store_smt.root(), &H256::zero());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let db_path = args.get(1).expect("args db_path not found");
    let listen_addr = args.get(2).expect("args listen_addr not found");
    let db = OptimisticTransactionDB::open_default(db_path).unwrap();
    let server = HttpServerBuilder::default()
        .build(listen_addr.parse::<SocketAddr>()?)
        .await?;
    let _handle = server.start(RpcServerImpl::new(db).into_rpc())?;
    println!("Server started at http://{}", listen_addr);
    futures::future::pending().await
}
