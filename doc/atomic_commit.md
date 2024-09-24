# Atomic commit in Qinhuai

For a database storage engine to be usable, it must support *durable* and *atomic* transactions. *Durability* means that data is saved to the disk. *Atomicity* means that all changes within a single transaction are either fully completed or not executed at all, as observed by the user. However, since real hardware takes time to write data to disk, a system crash or power failure during this process can result in intermediate states or corrupted files. Like mainstream databases, Qinhuai addresses this issue via *write-ahead logging* (WAL), a writing protocol which allows for data recovery.

## Hardware and file system assumptions

The effectiveness of any protocol depends on the guarantees provided by the underlying hardware and file system. In particular, Qinhuai assumes the following:

- At any time, all files which constitute the database are *modified by at most one process*, running a well-behaved version of Qinhuai.
- Opening/renaming a file never changes its size or content.
- Expanding/truncating a file causes its size to monotonously increase/decrease.
- Expanding/truncating a file never changes the existing/remaining content.
- When expanding a file, newly added bytes at the end of the file are being filled with random values, which are *very unlikely* to begin with a specific 64-bit magic number randomised at file creation and a valid 64-bit CRC checksum, at any point of time.
- Truncating a file to zero length is an atomic operation.
- Writing to a file never changes any content that does not overlap with the range of the write operation - [even if the range does not align to disk sector boundaries](https://www.sqlite.org/psow.html).
- Synchronising (i.e. `fsync`) a file guarantees that all data written till this point are saved to the disk.

Note that the atomicity of individual reads or writes, as well the preservation of their ordering, are *not* required. Some hardware (e.g. some AWS EC2 instances) supports atomic writes if they are aligned to disk sector boundaries, which can be taken advantage of to further improve write performance; due to my limited time, Qinhuai does *not* attempt such optimisation for now.

## How WAL works

The main database file, organised into 8KiB (default) pages, contains the tree structures and data tuples. When a new transaction is being committed, some pointers and tuples need to be changed. Instead of being written directly into the main database file, the changes are first appended into a separate WAL file. Sometime after the WAL file containing the change is successfully `fsync`ed and when there are no active readers, a *checkpoint* operation writes all WAL records into the main database file, then calls `fsync` on the main database file, then truncates WAL file to zero length.

At any time, if a WAL file is still present, it constitutes an indispensable part of the database. Any reader will (conceptually) consider WAL records as overwriting the main database file. The records in the WAL are stored alongside enough information so that checkpointing does not depend on the content of the main database itself; in particular, the first WAL record affecting a given page is preceded by the original full copy of that page. In this way, corrupted pages caused by a system crash *during checkpointing* can be fixed by re-applying the copy and the WAL records. This also ensures *idempotency*, which means no matter how many checkpoints have failed, as long as the next one succeeds, the database file will be put into the desired state.

Sometimes, variable-length attributes stored in the database will get too large to fit into a page. Qinhuai stores records larger than ~2KB (default) as separate files. In such situations, the separate files are `fsync`ed and have their checksums computed before any records owning them are appended to the WAL file. Conversely, when an attribute owning a separate file is being updated or deleted, the file is only deleted when the corresponding WAL record is being checkpointed. In this way, no reader can ever stumble upon a partially-written file or a broken owning reference, and the file deletion is guaranteed.

Another way to think about it is to consider the separate files as indispensable parts of the WAL or the main database file, depending on whether the owning reference has been checkpointed into the main database.
