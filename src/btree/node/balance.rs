/// Node balance.
#[derive(Debug)]
pub enum Balance {
	/// The node is balanced.
	Balanced,

	/// The node is overflowing.
	Overflow,

	/// The node is underflowing.
	/// 
	/// The boolean is `true` if the node is empty.
	Underflow(bool)
}