use std::{
	borrow::Borrow,
	ops::Deref
};
use crate::btree::{
	Storage,
	node::{
		Offset,
		ItemAccess
	}
};

/// Search in `sorted_items` for the item with the nearest key smaller or equal to the given one.
///
/// `sorted_items` is assumed to be sorted.
#[inline]
pub fn binary_search_min<'a, S: 'a + Storage, A: ItemAccess<'a, S> + ?Sized, Q: ?Sized>(
	sorted_items: &A,
	key: &Q
) -> Option<Offset> where S::Key: Borrow<Q>, Q: Ord, S::Key: 'a, S::Value: 'a {
	use crate::btree::node::item::Ref;
	if sorted_items.is_empty() || sorted_items.item(0.into()).unwrap().key().deref().borrow() > key {
		None
	} else {
		let mut i: Offset = 0.into();
		let mut j: Offset = (sorted_items.item_count() - 1).into();

		if sorted_items.item(i).unwrap().key().deref().borrow() <= key {
			return Some(j)
		}

		// invariants:
		// sorted_items[i].key <= key
		// sorted_items[j].key > key
		// j > i

		while j-i > 1 {
			let k = (i + j) / 2;

			if sorted_items.item(k).unwrap().key().deref().borrow() > key {
				j = k;
				// sorted_items[k].key > key --> sorted_items[j] > key
			} else {
				i = k;
				// sorted_items[k].key <= key --> sorted_items[i] <= key
			}
		}

		Some(i)
	}
}
