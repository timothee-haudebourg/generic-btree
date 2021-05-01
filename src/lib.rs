#![feature(generic_associated_types)]
#![feature(trait_alias)]

mod util;
pub mod btree;
pub mod map;

/// Default Slab-backed implementation.
pub mod slab;

pub use btree::{
	Storage,
	StorageMut
};
pub use map::Map;