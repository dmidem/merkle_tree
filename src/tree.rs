use crate::hasher::MerkleTreeHasher;

/// A Merkle tree data structure.
///
/// A Merkle tree is a binary tree where the leaves are hashes of the data items, and the internal nodes are
/// hashes of the concatenation of their child nodes. The root node of the tree is the hash of the entire
/// tree.
///
/// The `MerkleTree` structure is parameterized by a `MerkleTreeHasher` trait object, which specifies the
/// hash function used to generate the hashes in the tree.
///
/// # Examples
///
/// ```
/// use merkle_tree::{tree::MerkleTree, hasher::SdbmHasher};
///
/// type Tree = MerkleTree<SdbmHasher>;
///
/// let tree = Tree::from_data_items(["hello", "world"].iter());
///
/// let root_hash = tree.get_root().unwrap();
///
/// let proof = tree.get_proof(0).unwrap();
/// assert!(Tree::verify_proof(b"hello", root_hash, &proof));
/// assert!(!Tree::verify_proof(b"world", root_hash, &proof));
/// assert!(!Tree::verify_proof(b"fake", root_hash, &proof));
///
/// let proof = tree.get_proof(1).unwrap();
/// assert!(!Tree::verify_proof(b"hello", root_hash, &proof));
/// assert!(Tree::verify_proof(b"world", root_hash, &proof));
/// assert!(!Tree::verify_proof(b"fake", root_hash, &proof));
///
/// assert_eq!(tree.get_proof(2), None);
/// ```

#[derive(Debug)]
pub struct MerkleTree<Hasher: MerkleTreeHasher> {
    // The number of data items (i.e. leaves) in the tree.
    item_count: usize,

    // The number of levels in the tree.
    level_count: usize,

    // A flat array of the nodes in the tree, stored in reversed breadth-first order
    // (so the root node is the last element of the vector).
    nodes: Vec<Hasher::Hash>,
}

pub type Proof<Hasher> = Vec<(<Hasher as MerkleTreeHasher>::Hash, bool)>;

impl<Hasher: MerkleTreeHasher> MerkleTree<Hasher> {
    fn calc_tree_size(item_count: usize) -> (usize, usize) {
        let mut level_count = 0;
        let mut tree_node_count: usize = 0;
        let mut level_node_count = item_count;

        while level_node_count > 1 {
            level_count += 1;
            tree_node_count += level_node_count;
            level_node_count = (level_node_count + 1) >> 1;
        }

        // level_node_count is 1 or 0 here (0 only if the tree is empty 0 i.e. in case the data items
        // count is 0, 1 in all other cases)
        (
            tree_node_count + level_node_count,
            level_count + level_node_count,
        )
    }

    pub fn try_from_hash_items<Error, Items>(hash_items: Items) -> Result<Self, Error>
    where
        Items: IntoIterator<Item = Result<Hasher::Hash, Error>>,
        <Items as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let iter = hash_items.into_iter();
        let item_count = iter.len();

        let (node_count, level_count) = Self::calc_tree_size(item_count);

        let mut nodes = Vec::<Hasher::Hash>::with_capacity(node_count);

        // Add hash items (bottom level of the tree).
        for node in iter {
            node.map(|node| nodes.push(node))?;
        }

        // Add hashes for upper levels of the tree.
        let mut level_start_index = 0;
        while nodes.len() - level_start_index > 1 {
            let level_end_index = nodes.len();

            // Iterate through the current level and calculate the hashes
            // for the next level.
            for i in (level_start_index..level_end_index).into_iter().step_by(2) {
                let node_a = nodes[i];
                let node_b = if i + 1 < level_end_index {
                    nodes[i + 1]
                } else {
                    // Use the last node in the current level as the "right"
                    // child if the number of nodes is odd.
                    nodes[level_end_index - 1]
                };
                nodes.push(Hasher::concat(node_a, node_b));
            }

            level_start_index = level_end_index;
        }

        Ok(Self {
            item_count,
            level_count,
            nodes,
        })
    }

    pub fn from_hash_items<Items>(hash_items: Items) -> Self
    where
        Items: IntoIterator<Item = Hasher::Hash>,
        <Items as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        Self::try_from_hash_items(hash_items.into_iter().map(Ok::<_, ()>)).unwrap()
    }

    pub fn try_from_data_items<Error, Item, Items>(data_items: Items) -> Result<Self, Error>
    where
        Item: AsRef<[u8]>,
        Items: IntoIterator<Item = Result<Item, Error>>,
        <Items as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        Self::try_from_hash_items(
            data_items
                .into_iter()
                .map(|data| data.map(|data| Hasher::hash(data.as_ref()))),
        )
    }

    pub fn from_data_items<Item, Items>(data_items: Items) -> Self
    where
        Item: AsRef<[u8]>,
        Items: IntoIterator<Item = Item>,
        <Items as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        Self::try_from_data_items(data_items.into_iter().map(Ok::<_, ()>)).unwrap()
    }

    pub fn get_root(&self) -> Option<Hasher::Hash> {
        self.nodes.last().copied()
    }

    pub fn get_proof(&self, item_index: usize) -> Option<Proof<Hasher>> {
        if item_index >= self.item_count {
            return None;
        }
        let mut proof = Proof::<Hasher>::with_capacity(self.level_count - 1);

        let mut level_start_index = 0;
        let mut node_count = self.item_count; // node count in level
        let mut node_index = item_index; // node index in level

        while node_count > 1 {
            let sibling_node_index = (node_index ^ 1).min(node_count - 1);

            proof.push((
                self.nodes[level_start_index + sibling_node_index],
                sibling_node_index > node_index,
            ));

            level_start_index += node_count;
            node_count = (node_count + 1) >> 1;
            node_index >>= 1;
        }

        Some(proof)
    }

    fn calc_proof_hash(item_hash: Hasher::Hash, proof: &Proof<Hasher>) -> Hasher::Hash {
        proof
            .iter()
            .fold(item_hash, |proof_hash, (sibling_hash, is_right_sibling)| {
                if *is_right_sibling {
                    Hasher::concat(proof_hash, *sibling_hash)
                } else {
                    Hasher::concat(*sibling_hash, proof_hash)
                }
            })
    }

    pub fn verify_proof(item_data: &[u8], root_hash: Hasher::Hash, proof: &Proof<Hasher>) -> bool {
        let proof_hash = Self::calc_proof_hash(Hasher::hash(item_data), proof);
        proof_hash == root_hash
    }
}

#[test]
fn test() {
    const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur
        adipiscing elit, sed do eiusmod tempor incididunt ut labore et
        dolore magna aliqua. Ut enim ad minim veniam, quis nostrud
        exercitation ullamco laboris nisi ut aliquip ex ea commodo
        consequat. Duis aute irure dolor in reprehenderit in voluptate
        velit esse cillum dolore eu fugiat nulla pariatur. Excepteur
        sint occaecat cupidatat non proident, sunt in culpa qui
        officia deserunt mollit anim id est laborum.";

    let items = LOREM_IPSUM
        .split(|x: char| !x.is_alphabetic())
        .filter(|&x| !x.is_empty())
        .map(|word| word.as_bytes())
        .collect::<Vec<&[u8]>>();

    type Tree = MerkleTree<crate::hasher::SdbmHasher>;

    let tree = Tree::from_data_items(items.iter());

    let root_hash = tree.get_root().unwrap();

    for (item_index, item_data) in items.iter().enumerate() {
        let proof = tree.get_proof(item_index).unwrap();
        assert!(Tree::verify_proof(item_data, root_hash, &proof));
    }

    let wrong_item = "fake data".as_bytes();

    for (item_index, _) in items.iter().enumerate() {
        let proof = tree.get_proof(item_index).unwrap();
        assert!(!Tree::verify_proof(wrong_item, root_hash, &proof));
    }
}
