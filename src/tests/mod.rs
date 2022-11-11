use blake2b_rs::{Blake2b, Blake2bBuilder};
use rocksdb::DBVector;
use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, traits::Value,
    SparseMerkleTree, H256,
};

mod cf_store;
mod default_store;

#[derive(Default, Clone)]
pub struct Word(String);

impl Value for Word {
    fn to_h256(&self) -> H256 {
        if self.0.is_empty() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(self.0.as_bytes());
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}

impl From<DBVector> for Word {
    fn from(vec: DBVector) -> Self {
        Word(String::from_utf8(vec.to_vec()).expect("stored value is utf8"))
    }
}

impl AsRef<[u8]> for Word {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).personal(b"SMT").build()
}

pub type MemoryStoreSMT = SparseMerkleTree<Blake2bHasher, Word, DefaultStore<Word>>;
