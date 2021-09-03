# How to make your own BTree: Generic BTree implementation for Rust

This library provides a generic BTree implementation that you can use to make your own BTree data structure.
It abstracts away the tedious balancing operations and only require you to implement straight-forward node/item access functions.

## Usage

A BTree is defined as a set of nodes containing an indexed list of items.
This library tries to make the minimal assumptions on how nodes are accessed and stored,
so the first step in creating your own BTree data structure is to implement the
`Storage` trait describing the interface to the internal nodes:

```rust
impl generic_btree::Storage for MyBTree {
	type NodeRef<'r> where Self: 'r = MyNodeRef<'r>; // type of reference to a BTree node.
	type ItemRef<'r> where Self: 'r = MyItemRef<'r>; // type of reference to an item in a BTree node.
}
```