mod leaf;
mod internal;

pub use leaf::Leaf;
pub use internal::Internal;

pub enum Node<K, V> {
	Internal(Internal<K, V>),
	Leaf(Leaf<K, V>)
}