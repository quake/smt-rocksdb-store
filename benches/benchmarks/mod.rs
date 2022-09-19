use rand::{thread_rng, Rng};
use rocksdb::DBVector;
use sparse_merkle_tree::{traits::Value, H256};

pub mod cf_store;
pub mod default_store;

#[derive(Default, Clone)]
pub struct V([u8; 32]);

impl Value for V {
    fn to_h256(&self) -> H256 {
        self.0.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}

impl From<DBVector> for V {
    fn from(vec: DBVector) -> Self {
        let mut v = V::zero();
        v.0.copy_from_slice(&vec);
        v
    }
}

impl From<[u8; 32]> for V {
    fn from(v: [u8; 32]) -> Self {
        V(v)
    }
}

impl AsRef<[u8]> for V {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

pub fn random(rng: &mut impl Rng) -> [u8; 32] {
    let mut ret = [0u8; 32];
    rng.fill(&mut ret);
    ret
}

pub fn random_kvs(count: usize) -> Vec<(H256, V)> {
    let mut rng = thread_rng();
    (0..count)
        .map(|_| (random(&mut rng).into(), random(&mut rng).into()))
        .collect()
}
