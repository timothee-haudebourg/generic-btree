#![feature(generic_associated_types)]
#![feature(trait_alias)]

mod util;
mod btree;
mod map;

/// Default Slab-backed implementation.
mod slab;

pub use btree::{
	Storage,
	StorageMut
};
pub use map::Map;

// pub type BTreeMap<K, V> = map::BTree<slab::Storage<K, V>>;

// mod tests {
// 	use super::*;

// 	fn test() {
// 		let map: BTreeMap<usize, usize> = BTreeMap::new();
// 	}
// }