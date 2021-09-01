use std::cmp::Ordering;
use crate::btree::{
	KeyPartialOrd,
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
pub fn binary_search_min<'r, S: 'r + Storage, A: ItemAccess<S> + ?Sized, Q: ?Sized>(
	sorted_items: &'r A,
	key: &Q
) -> Option<(Offset, bool)> where S: KeyPartialOrd<Q> {
	if sorted_items.is_empty() || S::key_partial_cmp(&sorted_items.borrow_item(0.into()).unwrap(), key).map(Ordering::is_gt).unwrap_or(false) {
		None
	} else {
		let mut i: Offset = 0.into();
		let mut j: Offset = (sorted_items.item_count() - 1).into();

		let j_item = sorted_items.borrow_item(j).unwrap();
		if S::key_partial_cmp(&j_item, key).map(Ordering::is_le).unwrap_or(false) {
			let eq = S::key_partial_cmp(&j_item, key).map(Ordering::is_eq).unwrap_or(false);
			return Some((j, eq))
		}

		// invariants:
		// sorted_items[i].key <= key
		// sorted_items[j].key > key
		// j > i

		while j-i > 1 {
			let k = (i + j) / 2;

			if S::key_partial_cmp(&sorted_items.borrow_item(k).unwrap(), key).map(Ordering::is_gt).unwrap_or(false) {
				j = k;
				// sorted_items[k].key > key --> sorted_items[j] > key
			} else {
				i = k;
				// sorted_items[k].key <= key --> sorted_items[i] <= key
			}
		}

		let eq = S::key_partial_cmp(&sorted_items.borrow_item(i).unwrap(), key).map(Ordering::is_eq).unwrap_or(false);
		Some((i, eq))
	}
}