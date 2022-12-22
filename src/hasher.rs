trait SimpleHasher {
    type Hash: PartialEq + Copy + std::fmt::Debug;

    fn hash(data: &[u8]) -> Self::Hash;
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Hash64(u64);

impl Hash64 {
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for Hash64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x?}", self.0)
    }
}

#[derive(Debug)]
pub struct Djb2Hasher;

impl SimpleHasher for Djb2Hasher {
    type Hash = Hash64;

    fn hash(data: &[u8]) -> Hash64 {
        Hash64(data.iter().fold(5381, |hash, c| {
            (hash << 5).wrapping_add(hash).wrapping_add(*c as u64) // hash * 33 + c
        }))
    }
}

#[derive(Debug)]
pub struct SdbmHasher;

impl SimpleHasher for SdbmHasher {
    type Hash = Hash64;

    fn hash(data: &[u8]) -> Hash64 {
        Hash64(data.iter().fold(0, |hash, c| {
            (*c as u64)
                .wrapping_add(hash << 6)
                .wrapping_add(hash << 16)
                .wrapping_sub(hash)
        }))
    }
}

pub trait MerkleTreeHasher {
    type Hash: PartialEq + Copy + std::fmt::Debug;

    fn hash(data: &[u8]) -> Self::Hash;
    fn concat(hash1: Self::Hash, hash2: Self::Hash) -> Self::Hash;
}

impl<Hasher> MerkleTreeHasher for Hasher
where
    Hasher: SimpleHasher<Hash = Hash64>,
{
    type Hash = Hash64;

    fn hash(data: &[u8]) -> Hash64 {
        Self::hash(data)
    }

    fn concat(hash1: Hash64, hash2: Hash64) -> Hash64 {
        Self::hash(&((hash1.0 as u128) << 64 | (hash2.0 as u128)).to_le_bytes())
    }
}

#[test]
fn test_dbj2() {
    let hash1 = <Djb2Hasher as SimpleHasher>::hash("hello".as_bytes());
    let hash2 = <Djb2Hasher as SimpleHasher>::hash("world".as_bytes());

    assert_eq!(hash1, Hash64(0x0000_0031_0F92_3099));
    assert_eq!(hash2, Hash64(0x0000_0031_10A7_356D));
    assert_eq!(
        Djb2Hasher::concat(hash1, hash2),
        Hash64(0xE9B2_0141_B1A0_810A)
    );
}

#[test]
fn test_sdbm() {
    let hash1 = <SdbmHasher as SimpleHasher>::hash("hello".as_bytes());
    let hash2 = <SdbmHasher as SimpleHasher>::hash("world".as_bytes());

    assert_eq!(hash1, Hash64(0x66EB_1BB3_28D1_9932));
    assert_eq!(hash2, Hash64(0x75BE_975B_F7E3_AEB2));
    assert_eq!(
        SdbmHasher::concat(hash1, hash2),
        Hash64(0x8108_4122_AFDB_AAE4)
    );
}
