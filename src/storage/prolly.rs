//! # Prolly trees
//!
//! A Prolly tree is an N-ary search tree typically with a high fanout. It has a structure similar
//! to that of B+ trees.
//!
//! Unlike B+ trees, it does not balance itself through rotations; instead, it uses a pseudo-random
//! value seeded on `(height, key)` pairs and a variable probability dependent on the current node
//! size to decide node boundaries. This allows for more predictable node boundaries (in particular,
//! two trees containing the same set of keys will be structurally identical; this property is
//! called *unicity*), which is crucial for amortized near-O(d) diffing between trees.

use super::paging;
use std::ops;

/// # Prolly tree interface
pub trait Tree<Store: paging::Store> {
  /// The type of cursors used by this tree.
  type Cursor: Cursor<Store>;

  /// Returns a copy of the value corresponding to the key.
  fn get(&self, store: &mut Store, key: &[u8]) -> Option<Box<[u8]>>;

  /// Inserts or updates a key-value pair in the map. Returns whether the key was present.
  fn insert(&mut self, store: &mut Store, key: &[u8], value: &[u8]) -> bool;

  /// Removes a key-value pair from the map. Returns whether the key was present.
  fn remove(&mut self, store: &mut Store, key: &[u8]) -> bool;

  /// Returns a [`Cursor`] pointing at the gap after the greatest key smaller than the given bound.
  fn upper_bound(&self, store: &mut Store, bound: ops::Bound<&[u8]>) -> Self::Cursor;

  /// Returns a [`Cursor`] pointing at the gap before the smallest key greater than the given bound.
  fn lower_bound(&self, store: &mut Store, bound: ops::Bound<&[u8]>) -> Self::Cursor;

  // TODO: diffing
}

/// # Prolly tree cursor interface
///
/// A [`Cursor`] is like an iterator, except that it can freely seek back-and-forth.
///
/// Cursors always point to a gap between two elements in the map, and can operate on the two
/// immediately adjacent elements.
pub trait Cursor<Store: paging::Store> {
  /// Advances the cursor to the next gap, returning the key and value of the element that it moved
  /// over. If the cursor is already at the end of the map then `None` is returned and the cursor is
  /// not moved.
  fn next(&mut self, store: &mut Store) -> Option<(&[u8], &[u8])>;

  /// Advances the cursor to the previous gap, returning the key and value of the element that it
  /// moved over. If the cursor is already at the start of the map then `None` is returned and the
  /// cursor is not moved.
  fn prev(&mut self, store: &mut Store) -> Option<(&[u8], &[u8])>;

  /// Returns a reference to the key and value of the next element without moving the cursor.
  /// If the cursor is at the end of the map then `None` is returned.
  fn peek_next(&mut self, store: &mut Store) -> Option<(&[u8], &[u8])>;

  /// Returns a reference to the key and value of the previous element without moving the cursor.
  /// If the cursor is at the start of the map then `None` is returned.
  fn peek_prev(&mut self, store: &mut Store) -> Option<(&[u8], &[u8])>;
}

/// # Prolly tree policy interface
///
/// A Prolly tree policy specifies the boundary decision and content hash functions for a Prolly
/// tree.
///
/// A good policy is crucial for performance. See method-specific documentation for more details.
pub trait Policy {
  /// The boundary decision function. Returns `true` iff the node should be split here.
  ///
  /// Implementations should first produce a pseudo-random value seeded by the `(height, key)` pair,
  /// then return `true` iff the value is less than a certain threshold `thres`, which must be
  /// monotonously non-decreasing as `size` (the current node size) increases.
  ///
  /// - A constant `thres` prevents cascading splits, but also results in a geometric distribution
  ///   of node sizes, which has a long tail (i.e. there can be very large nodes) causing degraded
  ///   read performance.
  /// - A steep increase in `thres` results in more uniform node sizes, but also increases the
  ///   chances of very long cascading splits, causing degraded write performance.
  fn boundary_decision(&self, height: usize, key: &[u8], size: usize) -> bool;

  /// The content hash function. Returns a *collision-resistant* hash of given `content`.
  ///
  /// This will be called on either the values in leaf nodes, or the `(key, hash)` pairs in
  /// internal nodes.
  fn content_hash(&self, content: &[u8]) -> Box<[u8]>;
}

/// # Standard implementation for [`Tree`]
///
/// ## Implementation notes
///
/// Invariants maintained by all methods:
///
/// - All leaf nodes are at the same depth.
///
/// - All internal nodes have the same key as its leftmost child.
///
/// - For the `i`-th entry in a node with height `height` and child key list `keys`,
///   `boundary_decision(height, keys[i], i + 1) == true` iff `i + 1 == size`.
///   
///   - Note that the first three invariants uniquely determine the tree's structure from a list of
///     keys: imagine constructing the tree layer-by-layer starting from the leaves. In the first
///     layer, traverse the list of keys, adding keys to the current node until `boundary_decision`
///     returns `true`, at which point a new node is started at the next key. Once all keys are
///     grouped into nodes, use the first key in each group as the node's key. Repeat this process
///     until only one node remains in a layer.
pub struct BasicTree<Store: paging::Store, Policy: self::Policy> {
  _store: std::marker::PhantomData<Store>,
  _policy: std::marker::PhantomData<Policy>,
  // TODO: implement
}

pub struct BasicNode<Store: paging::Store> {
  _store: std::marker::PhantomData<Store>,
  // TODO: implement
}
