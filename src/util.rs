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
pub fn binary_search_min<'r, S: Storage, A: ItemAccess<S> + ?Sized, Q: ?Sized>(
	sorted_items: &'r A,
	key: &Q
) -> Option<(Offset, bool)> where S::ItemRef<'r>: PartialOrd<Q> {
	if sorted_items.is_empty() || sorted_items.borrow_item(0.into()).unwrap() > *key {
		None
	} else {
		let mut i: Offset = 0.into();
		let mut j: Offset = (sorted_items.item_count() - 1).into();

		if sorted_items.borrow_item(j).unwrap() <= *key {
			let eq = sorted_items.borrow_item(j).unwrap() == *key;
			return Some((j, eq))
		}

		// invariants:
		// sorted_items[i].key <= key
		// sorted_items[j].key > key
		// j > i

		while j-i > 1 {
			let k = (i + j) / 2;

			if sorted_items.borrow_item(k).unwrap() > *key {
				j = k;
				// sorted_items[k].key > key --> sorted_items[j] > key
			} else {
				i = k;
				// sorted_items[k].key <= key --> sorted_items[i] <= key
			}
		}

		let eq = sorted_items.borrow_item(i).unwrap() == *key;
		Some((i, eq))
	}
}