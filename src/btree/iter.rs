use std::{
	iter::{
		FusedIterator,
		ExactSizeIterator,
		DoubleEndedIterator
	},
	ops::{
		RangeBounds,
		Bound
	}
};
use super::{
	KeyPartialOrd,
	Storage,
	StorageMut,
	Address,
	node::item::{
		Read
	}
};

/// B-Tree items iterator.
pub struct Iter<'a, S> {
	/// BTree reference.
	storage: &'a S,

	/// Address of the next item.
	addr: Option<Address>,

	/// Address of the iterator's end.
	end: Option<Address>,

	/// Number of items left to iterate.
	len: usize
}

impl<'a, S: Storage> Iter<'a, S> {
	#[inline]
	pub(crate) fn new(storage: &'a S) -> Self {
		let addr = storage.first_item_address();
		let len = storage.len();
		Self {
			storage,
			addr,
			end: None,
			len
		}
	}
}

impl<'a, S: Storage> Iterator for Iter<'a, S> {
	type Item = S::ItemRef<'a>;

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self.addr {
			Some(addr) => {
				if self.len > 0 {
					self.len -= 1;

					let item = self.storage.item(addr).unwrap();
					self.addr = self.storage.next_item_address(addr);
					Some(item)
				} else {
					None
				}
			},
			None => None
		}
	}
}

impl<'a, S: Storage> FusedIterator for Iter<'a, S> { }
impl<'a, S: Storage> ExactSizeIterator for Iter<'a, S> { }

impl<'a, S: Storage> DoubleEndedIterator for Iter<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.len > 0 {
			let addr = match self.end {
				Some(addr) =>  self.storage.previous_item_address(addr).unwrap(),
				None => self.storage.last_item_address().unwrap()
			};

			self.len -= 1;

			let item = self.storage.item(addr).unwrap();
			self.end = Some(addr);
			Some(item)
		} else {
			None
		}
	}
}

/// An owning iterator over the entries of a `Storage`.
///
/// This `struct` is created by the [`into_iter`] method on [`Storage`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
pub struct IntoIter<S> {
	/// The tree.
	btree: S,

	/// Address of the next item, or the last valid address.
	addr: Option<Address>,

	/// Address following the last item.
	end: Option<Address>,

	/// Number of remaining items.
	len: usize
}

impl<S: Storage> IntoIter<S> {
	#[inline]
	pub fn new(btree: S) -> Self {
		let addr = btree.first_item_address();
		let len = btree.len();
		IntoIter {
			btree,
			addr,
			end: None,
			len
		}
	}
}

impl<S: StorageMut> FusedIterator for IntoIter<S> where for<'a> S::ItemRef<'a>: Read<S> { }
impl<S: StorageMut> ExactSizeIterator for IntoIter<S> where for<'a> S::ItemRef<'a>: Read<S> { }

impl<S: StorageMut> Iterator for IntoIter<S> where for<'a> S::ItemRef<'a>: Read<S> {
	type Item = S::Item;

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self.addr {
			Some(addr) => {
				if self.len > 0 {
					self.len -= 1;
		
					let item = unsafe {
						// this is safe because the item at `self.end` exists and is never touched again.
						let item = self.btree.item(addr).unwrap();
						item.read()
					};
		
					if self.len > 0 {
						self.addr = self.btree.next_back_address(addr); // an item address is always followed by a valid address.
		
						while let Some(addr) = self.addr {
							if addr.offset < self.btree.node(addr.id).unwrap().item_count() {
								break // we have found an item address.
							} else {
								self.addr = self.btree.next_back_address(addr);
		
								// we have gove through every item of the node, we can release it.
								let node = self.btree.release_node(addr.id);
								node.forget(); // do not call `drop` on the node since items have been moved.
							}
						}
					} else {
						// cleanup.
						if self.end.is_some() {
							while self.addr != self.end {
								let addr = self.addr.unwrap();
								self.addr = self.btree.next_back_address(addr);
	
								if addr.offset >= self.btree.node(addr.id).unwrap().item_count() {
									let node = self.btree.release_node(addr.id);
									node.forget(); // do not call `drop` on the node since items have been moved.
								}
							}
						}

						if let Some(addr) = self.addr {
							let mut id = Some(addr.id);
							while let Some(node_id) = id {
								let node = self.btree.release_node(node_id);
								id = node.parent();
								node.forget(); // do not call `drop` on the node since items have been moved.
							}
						}
					}
		
					Some(item)
				} else {
					None
				}
			},
			None => None
		}
	}
}

impl<S: StorageMut> DoubleEndedIterator for IntoIter<S> where for<'a> S::ItemRef<'a>: Read<S> {
	fn next_back(&mut self) -> Option<S::Item> {
		if self.len > 0 {
			let addr = match self.end {
				Some(mut addr) => {
					addr = self.btree.previous_front_address(addr).unwrap();
					while addr.offset.is_before() {
						let id = addr.id;
						addr = self.btree.previous_front_address(addr).unwrap();
	
						// we have gove through every item of the node, we can release it.
						let node = self.btree.release_node(id);
						node.forget(); // do not call `drop` on the node since items have been moved.
					}

					addr
				},
				None => self.btree.last_item_address().unwrap()
			};

			self.len -= 1;

			let item = unsafe {
				// this is safe because the item at `self.end` exists and is never touched again.
				let item = self.btree.item(addr).unwrap();
				item.read()
			};

			self.end = Some(addr);

			if self.len == 0 {
				// cleanup.
				while self.addr != self.end {
					let addr = self.addr.unwrap();
					self.addr = self.btree.next_back_address(addr);

					if addr.offset >= self.btree.node(addr.id).unwrap().item_count() {
						let node = self.btree.release_node(addr.id);
						node.forget(); // do not call `drop` on the node since items have been moved.
					}
				}

				if let Some(addr) = self.addr {
					let mut id = Some(addr.id);
					while let Some(node_id) = id {
						let node = self.btree.release_node(node_id);
						id = node.parent();
						node.forget(); // do not call `drop` on the node since items have been moved.
					}
				}
			}

			Some(item)
		} else {
			None
		}
	}
}

/// B-Tree mutable items iterator.
/// 
/// Note that it is a logical error to
/// mutate the items in a ways that changes their relative ordering.
pub struct IterMut<'a, S> {
	/// BTree mutable reference.
	storage: &'a mut S,

	/// Address of the next item.
	addr: Option<Address>,

	end: Option<Address>,

	len: usize
}

impl<'a, S: StorageMut> IterMut<'a, S> {
	#[inline]
	pub(crate) fn new(storage: &'a mut S) -> Self {
		let addr = storage.first_item_address();
		let len = storage.len();
		Self {
			storage,
			addr,
			end: None,
			len
		}
	}
}

impl<'a, S: StorageMut> Iterator for IterMut<'a, S> {
	type Item = S::ItemMut<'a>;

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	#[inline]
	fn next(&mut self) -> Option<S::ItemMut<'a>> {
		match self.addr {
			Some(addr) => {
				if self.len > 0 {
					self.len -= 1;

					self.addr = self.storage.next_item_address(addr);

					// this is safe because only one mutable reference to the same item can be emitted.
					unsafe {
						let ptr = self.storage as *mut S;
						let storage: &'a mut S = &mut *ptr;
						storage.item_mut(addr)
					}
				} else {
					None
				}
			},
			None => None
		}
	}
}

impl<'a, S: StorageMut> FusedIterator for IterMut<'a, S> { }
impl<'a, S: StorageMut> ExactSizeIterator for IterMut<'a, S> { }

impl<'a, S: StorageMut> DoubleEndedIterator for IterMut<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.len > 0 {
			let addr = match self.end {
				Some(addr) =>  self.storage.previous_item_address(addr).unwrap(),
				None => self.storage.last_item_address().unwrap()
			};

			self.len -= 1;

			// this is safe because only one mutable reference to the same item can be emitted.
			unsafe {
				let ptr = self.storage as *mut S;
				let storage: &'a mut S = &mut *ptr;
				storage.item_mut(addr)
			}
		} else {
			None
		}
	}
}

fn is_valid_range<T, R>(range: &R) -> bool where T: Ord + ?Sized, R: RangeBounds<T> {
	match (range.start_bound(), range.end_bound()) {
		(Bound::Included(start), Bound::Included(end)) => start <= end,
		(Bound::Included(start), Bound::Excluded(end)) => start <= end,
		(Bound::Included(_), Bound::Unbounded) => true,
		(Bound::Excluded(start), Bound::Included(end)) => start <= end,
		(Bound::Excluded(start), Bound::Excluded(end)) => start < end,
		(Bound::Excluded(_), Bound::Unbounded) => true,
		(Bound::Unbounded, _) => true
	}
}

/// Range iterator.
pub struct Range<'a, S> {
	/// The tree reference.
	btree: &'a S,

	/// Address of the next item or last back address.
	addr: Address,

	end: Address
}

impl<'a, S: Storage> Range<'a, S> {
	pub(crate) fn new<T, R>(btree: &'a S, range: R) -> Self where T: Ord + ?Sized, R: RangeBounds<T>, S: KeyPartialOrd<T> {
		if !is_valid_range(&range) {
			panic!("Invalid range")
		}

		let addr = match range.start_bound() {
			Bound::Included(start) => {
				match btree.address_of(start) {
					Ok(addr) => addr,
					Err(addr) => addr
				}
			},
			Bound::Excluded(start) => {
				match btree.address_of(start) {
					Ok(addr) => btree.next_item_or_back_address(addr).unwrap(),
					Err(addr) => addr
				}
			},
			Bound::Unbounded => btree.first_back_address()
		};

		let end = match range.end_bound() {
			Bound::Included(end) => {
				match btree.address_of(end) {
					Ok(addr) => btree.next_item_or_back_address(addr).unwrap(),
					Err(addr) => addr
				}
			},
			Bound::Excluded(end) => {
				match btree.address_of(end) {
					Ok(addr) => addr,
					Err(addr) => addr
				}
			},
			Bound::Unbounded => btree.first_back_address()
		};
		
		Range {
			btree,
			addr,
			end
		}
	}
}

impl<'a, S: Storage> Iterator for Range<'a, S> {
	type Item = S::ItemRef<'a>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.addr != self.end {
			let item = self.btree.item(self.addr).unwrap();
			self.addr = self.btree.next_item_or_back_address(self.addr).unwrap();
			Some(item)
		} else {
			None
		}
	}
}

impl<'a, S: Storage> FusedIterator for Range<'a, S> { }

impl<'a, S: Storage> DoubleEndedIterator for Range<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.addr != self.end {
			let addr = self.btree.previous_item_address(self.addr).unwrap();
			let item = self.btree.item(addr).unwrap();
			self.end = addr;
			Some(item)
		} else {
			None
		}
	}
}

/// Mutable range iterator.
/// 
/// Note that it is a logical error to mutate the items
/// in a ways that changes their relative ordering.
pub struct RangeMut<'a, S> {
	/// The tree reference.
	btree: &'a mut S,

	/// Address of the next item or last back address.
	addr: Address,

	end: Address
}

impl<'a, S: StorageMut> RangeMut<'a, S> {
	pub(crate) fn new<T, R>(btree: &'a mut S, range: R) -> Self where T: Ord + ?Sized, R: RangeBounds<T>, S: KeyPartialOrd<T> {
		if !is_valid_range(&range) {
			panic!("Invalid range")
		}

		let addr = match range.start_bound() {
			Bound::Included(start) => {
				match btree.address_of(start) {
					Ok(addr) => addr,
					Err(addr) => addr
				}
			},
			Bound::Excluded(start) => {
				match btree.address_of(start) {
					Ok(addr) => btree.next_item_or_back_address(addr).unwrap(),
					Err(addr) => addr
				}
			},
			Bound::Unbounded => btree.first_back_address()
		};

		let end = match range.end_bound() {
			Bound::Included(end) => {
				match btree.address_of(end) {
					Ok(addr) => btree.next_item_or_back_address(addr).unwrap(),
					Err(addr) => addr
				}
			},
			Bound::Excluded(end) => {
				match btree.address_of(end) {
					Ok(addr) => addr,
					Err(addr) => addr
				}
			},
			Bound::Unbounded => btree.first_back_address()
		};
		
		RangeMut {
			btree,
			addr,
			end
		}
	}
}

impl<'a, S: StorageMut> Iterator for RangeMut<'a, S> {
	type Item = S::ItemMut<'a>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.addr != self.end {
			let addr = self.addr;
			self.addr = self.btree.next_item_or_back_address(addr).unwrap();

			// this is safe because only one mutable reference to the same item can be emitted.
			unsafe {
				let btree: &'a mut S = std::ptr::read(&self.btree);
				let item = btree.item_mut(addr).unwrap();
				Some(item)
			}
		} else {
			None
		}
	}
}

impl<'a, S: StorageMut> FusedIterator for RangeMut<'a, S> { }

impl<'a, S: StorageMut> DoubleEndedIterator for RangeMut<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.addr != self.end {
			let addr = self.btree.previous_item_address(self.addr).unwrap();

			// this is safe because only one mutable reference to the same item can be emitted.
			unsafe {
				let btree: &'a mut S = std::ptr::read(&self.btree);
				let item = btree.item_mut(addr).unwrap();
				Some(item)
			}
		} else {
			None
		}
	}
}

pub(crate) struct DrainFilterInner<'a, S> {
	/// The tree reference.
	btree: &'a mut S,

	/// Address of the next item, or last valid address.
	addr: Address,

	len: usize
}

impl<'a, S: StorageMut> DrainFilterInner<'a, S> {
	#[inline]
	pub fn new(btree: &'a mut S) -> Self {
		let addr = btree.first_back_address();
		let len = btree.len();
		DrainFilterInner {
			btree,
			addr,
			len
		}
	}

	#[inline]
	pub fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.len))
	}

	#[inline]
	pub fn next<F>(&mut self, pred: &mut F) -> Option<S::Item> where F: FnMut(S::ItemMut<'_>) -> bool {
		loop {
			let remove = self.btree.item_mut(self.addr).map(|item| (*pred)(item));

			eprintln!("remove: {:?}", remove);

			match remove {
				Some(true) => {
					let (item, next_addr) = self.btree.remove_at(self.addr).unwrap();
					self.len -= 1;
					self.addr = next_addr;
					return Some(item)
				},
				Some(false) => {
					self.len -= 1;
					self.addr = self.btree.next_item_or_back_address(self.addr).unwrap();
				},
				None => return None
			}
		}
	}

	#[inline]
	pub fn next_consume<F>(&mut self, mut pred: F) -> Option<S::Item> where F: FnMut(S::ItemMut<'_>) -> bool {
		eprintln!("next_consume");
		loop {
			let remove = self.btree.item_mut(self.addr).map(|item| pred(item));

			eprintln!("remove: {:?}", remove);

			match remove {
				Some(true) => {
					let (item, next_addr) = self.btree.remove_at(self.addr).unwrap();
					self.len -= 1;
					self.addr = next_addr;
					return Some(item)
				},
				Some(false) => {
					self.len -= 1;
					self.addr = self.btree.next_item_or_back_address(self.addr).unwrap();
				},
				None => return None
			}
		}
	}
}

/// Drain filter iterator.
pub struct DrainFilter<'a, S: StorageMut, F> where F: FnMut(S::ItemMut<'_>) -> bool {
	pred: F,

	inner: DrainFilterInner<'a, S>
}

impl<'a, S: StorageMut, F> DrainFilter<'a, S, F> where F: FnMut(S::ItemMut<'_>) -> bool {
	#[inline]
	pub(crate) fn new(btree: &'a mut S, pred: F) -> Self {
		DrainFilter {
			pred,
			inner: DrainFilterInner::new(btree)
		}
	}
}

impl<'a, S: StorageMut, F> FusedIterator for DrainFilter<'a, S, F> where F: FnMut(S::ItemMut<'_>) -> bool { }

impl<'a, S: StorageMut, F> Iterator for DrainFilter<'a, S, F> where F: FnMut(S::ItemMut<'_>) -> bool {
	type Item = S::Item;

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next(&mut self.pred)
	}
}

impl<'a, S: StorageMut, F> Drop for DrainFilter<'a, S, F> where F: FnMut(S::ItemMut<'_>) -> bool {
	#[inline]
	fn drop(&mut self) {
		loop {
			if self.next().is_none() {
				break
			}
		}
	}
}