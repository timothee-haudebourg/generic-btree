#![feature(generic_associated_types)]
#![feature(trait_alias)]

mod util;
mod btree;
pub mod map;

#[cfg(feature="dot")]
pub mod dot;

// /// Default Slab-backed implementation.
pub mod slab;

pub use btree::*;

pub use map::Map;