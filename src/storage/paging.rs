//! # Paging and write-ahead logging
//!
//! This module provides buffer pool managers for slotted page files and write-ahead logs.

#![doc = include_str!("../../doc/file_format.md")]

use super::vfs;

/// # Slotted page store
///
/// A slotted page store manages a collection of fixed-size pages indexed by 64-bit unsigned
/// integers.
///
/// Each page contains a number of records, which are simply byte arrays. They can be used to
/// store e.g. keys and child pointers in B+ tree internal nodes, or keys and values in leaf nodes,
/// among other things.
///
/// It is also responsible for page allocation through the use of a freelist.
pub trait Store {
  /// The type of files used to store pages.
  type File: vfs::File;

  // /// Obtains a page from the store.
  // fn get(&mut self, page_id: u64) -> Result<Self::Page, <Self::File as vfs::File>::Error>;

  // /// Writes a page to the store.
  // fn write(&mut self, page_id: u64, slot_id: u16) -> Result<(), <Self::File as vfs::File>::Error>;

  // /// Allocates a new page in the store.
  // fn allocate(&mut self) -> Result<u64, <Self::File as vfs::File>::Error>;

  // /// Deallocates a page in the store.
  // fn deallocate(&mut self, id: u64) -> Result<(), <Self::File as vfs::File>::Error>;
}
