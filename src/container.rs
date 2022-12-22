use crate::{
    hasher::MerkleTreeHasher,
    tree::{MerkleTree, Proof},
};

#[derive(Debug)]
pub struct MerkleTreeContainer<Hasher: MerkleTreeHasher, Item: AsRef<[u8]>> {
    tree: MerkleTree<Hasher>,
    items: Vec<Item>,
}

impl<Hasher: MerkleTreeHasher, Item: AsRef<[u8]>> MerkleTreeContainer<Hasher, Item> {
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            tree: MerkleTree::<Hasher>::from_data_items(items.iter()),
            items,
        }
    }

    pub fn get_root(&self) -> Option<Hasher::Hash> {
        self.tree.get_root()
    }

    pub fn get_item(&self, item_index: usize) -> Option<(&Item, Proof<Hasher>)> {
        Some((
            self.items.get(item_index)?,
            self.tree.get_proof(item_index)?,
        ))
    }
}
