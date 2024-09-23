/// # Page store
///
/// A page store manages a collection of fixed-size pages indexed by 64-bit unsigned integers.
/// In persistent implementations, it also manages an in-memory buffer pool, changes to which
/// can be either discarded or written back to disk when requested.
///
/// See: <https://db.cs.cmu.edu/mmap-cidr2022/>
pub trait Store {
  // TODO: speficy operations
}
