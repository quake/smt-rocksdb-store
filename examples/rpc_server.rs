use std::env;
use std::net::SocketAddr;

use jsonrpsee::core::{async_trait, Error};
use jsonrpsee::http_server::HttpServerBuilder;
use jsonrpsee::proc_macros::rpc;

use rocksdb::{prelude::Open, DBVector, OptimisticTransactionDB};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use smt_rocksdb_store::default_store::DefaultStore;
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

type DefaultStoreSMT<'a, T, W> = SparseMerkleTree<Blake2bHasher, SmtValue, DefaultStore<'a, T, W>>;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "update_all")]
    async fn update_all(&self, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error>;

    #[method(name = "merkle_proof")]
    async fn merkle_proof(&self, keys: Vec<SmtKey>) -> Result<SmtProof, Error>;
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
    async fn update_all(&self, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error> {
        let kvs: Vec<(H256, SmtValue)> = kvs.into_iter().map(|(k, v)| (k.0.into(), v)).collect();

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt =
            DefaultStoreSMT::new_with_store(DefaultStore::new(&tx)).unwrap();
        rocksdb_store_smt.update_all(kvs).expect("update_all error");
        tx.commit().expect("db commit error");
        Ok(SmtRoot(rocksdb_store_smt.root().clone().into()))
    }

    async fn merkle_proof(&self, keys: Vec<SmtKey>) -> Result<SmtProof, Error> {
        let keys: Vec<H256> = keys.into_iter().map(|k| k.0.into()).collect();
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt =
            DefaultStoreSMT::new_with_store(DefaultStore::<_, ()>::new(&snapshot)).unwrap();
        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())
            .expect("merkle_proof error");
        Ok(SmtProof(proof.compile(keys).expect("compile error").0))
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
