# How to make your own BTree: Generic BTree implementation for Rust

<table><tr>
  <td><a href="https://docs.rs/generic-btree">Documentation</a></td>
  <td><a href="https://crates.io/crates/generic-btree">Crate informations</a></td>
  <td><a href="https://github.com/timothee-haudebourg/generic-btree">Repository</a></td>
</tr></table>

This library provides a generic BTree implementation that you can use to make your own BTree data structure.
It abstracts away the tedious balancing operations and only require you to implement straight-forward node/item access functions.

## Usage

A BTree is defined as a set of nodes containing an indexed list of items.

```text
                                  ┌────────────────┐
                                  │ internal node  │
                                  │┌────────┐ ┌───┐│
                       ┌───────── ││ item 0 │ │ 1 ││ ──────────┐
                       │          │└────────┘ └───┘│           │
                       │          └────────────────┘           │
                       │                   │                   │
              ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
              │ leaf node       │ │ leaf node       │ │ leaf node       │
              │┌───┐ ┌───┐ ┌───┐│ │┌───┐ ┌───┐ ┌───┐│ │┌───┐ ┌───┐ ┌───┐│
              ││ 0 │ │ 1 │ │ 2 ││ ││ 0 │ │ 1 │ │ 2 ││ ││ 0 │ │ 1 │ │ 2 ││
              │└───┘ └───┘ └───┘│ │└───┘ └───┘ └───┘│ │└───┘ └───┘ └───┘│
              └─────────────────┘ └─────────────────┘ └─────────────────┘
```

This library tries to make the minimal assumptions on the internal data structure by defining a collection of traits that must be implemented:

- `Storage`: defines how nodes are stored and accessed,
- `InternalRef`: defines how internal nodes are accessed,
- `LeafRef`: defines how leaf nodes are accessed,
- `ItemRef`: defines how items are accessed.

Each item in the tree is identified by an `Address` composed
of a node id, and an item index in the node.
The `Storage` trait provides the necessary functions to
access nodes by id, and defines what types are used as
node and item references:

```rust
impl generic_btree::Storage for MyBTree {
	// type of reference to an item in a BTree node.
	type ItemRef<'r> where Self: 'r = MyItemRef<'r>;

	// type of reference to a BTree leaf node.
	type LeafRef<'r> where Self: 'r = MyLeafRef<'r>;

	// type of reference to an internal BTree node.
	type InternalRef<'r> where Self: 'r = MyInternalRef<'r>;

	fn root(&self) -> Option<usize> {
		self.root
	}

	fn len(&self) -> usize {
		self.len
	}

	/// Returns the node with the given `id`.
	fn node(&self, id: usize) -> Option<btree::node::Ref<'_, Self>> {
		...
	}
}
```

## Items ordering

Up until now, no order between items has been defined.
Since there is no concrete types for items because they are
indirectly accessed through the `Storage::ItemRef` type,
the standard `PartialOrd` trait cannot be used to define such order.
Instead, this library defines the `ItemPartialOrd` trait that must
be implemented by the storage.

## Key ordering

An usual way to access a BTree is to fetch items matching a given key.
To this end, this library defines a dedicated comparison trait,
`KeyPartialOrd`, similar to the `ItemPartialOrd` trait but for key comparison. 
This allows us to define the, for instance, the `Storage::get` function to return a reference to the item matching the provided key:

```rust
/// Returns a reference to the item identified by the supplied key.
fn get<Q: ?Sized>(&self, key: &Q) -> Option<Self::ItemRef<'_>> where Self: KeyPartialOrd<Q>;
```

## Mutable BTree

The traits defined until now only specify how the BTree is accessed,
but not how it is modified. This is simply done by implementing the following traits on the corresponding types:

- `StorageMut`: defines how nodes are allocated, inserted and removed from the underlying storage,
- `InternalMut`: defines how items are inserted and removed from an internal node,
- `LeafRef`: defines how items are inserted and removed from a leaf node,
- `ItemMut`: defines how items are modified.

In addition, to be able to directly insert a node or item,
some concrete types must be defined:

- `Item`: the type representing an item being inserted or removed from a node,
- `Leaf`: the type representing a leaf node being inserted or removed from the storage,
- `Internal`: the type representing an internal node being inserted or removed from the storage.