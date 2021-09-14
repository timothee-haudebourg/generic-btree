#![feature(generic_associated_types)]
#![feature(trait_alias)]

mod btree;
mod util;

/// Map components.
pub mod map;

/// Graphviz DOT language export features.
#[cfg(feature = "dot")]
pub mod dot;

/// Default Slab-backed implementation.
pub mod slab;

pub use btree::*;

pub use map::Map;
