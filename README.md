# Simple implementation of Merkle tree data structure.

Merkle tree is a binary tree where the leaves are hashes of the data items, and the internal nodes are
hashes of the concatenation of their child nodes. The root node of the tree is the hash of the entire
tree.

See the following links for more information on Merkle tree and how it is used:

https://en.wikipedia.org/wiki/Merkle_tree:

https://en.bitcoin.it/wiki/Merged_mining_specification#Merkle_Branch

http://bittorrent.org/beps/bep_0030.html

## Example of usage:

```rust
use merkle_tree::{tree::MerkleTree, hasher::SdbmHasher};

type Tree = MerkleTree<SdbmHasher>;

let tree = Tree::from_data_items(["hello", "world"].iter());

let root_hash = tree.get_root().unwrap();

let proof = tree.get_proof(0).unwrap();
assert!(Tree::verify_proof(b"hello", root_hash, &proof));
assert!(!Tree::verify_proof(b"world", root_hash, &proof));
assert!(!Tree::verify_proof(b"fake", root_hash, &proof));

let proof = tree.get_proof(1).unwrap();
assert!(!Tree::verify_proof(b"hello", root_hash, &proof));
assert!(Tree::verify_proof(b"world", root_hash, &proof));
assert!(!Tree::verify_proof(b"fake", root_hash, &proof));

assert_eq!(tree.get_proof(2), None);
```
