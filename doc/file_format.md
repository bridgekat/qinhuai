# Database file format

Like many relational databases, most of the records in Qinhuai are stored in the leaves of tree structures. Each internal node or leaf node takes up a single 8KiB (default) *page* in a disk file. If some node grows larger than a page, it will be splitted into two nodes, as part of a *re-balancing* process. If some non-root node shrinks smaller than half of a page, more records will be moved to this node, as part of a *re-balancing* process.

Keys and records larger than ~2KB (default) are stored as separate files, in order to ensure that each node is able to contain at least 4 records or 5 child pointers. Pages are at least half-full, so each node contains at least 2 records or 3 child pointers. In most cases, however, a node can contain *hundreds* of small keys and child pointers.

To ensure performance and atomicity, Qinhuai uses *write-ahead logging* (WAL) for most transactions. This puts a part of the database state into a separate WAL file, which needs to be periodically *checkpointed* into the main database file.

## The main database file

The main database file is simply an array of pages. Page `0` is the database header page. All others are either free pages or node pages. Page `1` is the root node of the *schema table*, which contains pointers to the root node pages of all other tables and indices, as well as information necessary for their interpretations.

### The database header page

The header page contains a single 24-byte header.

| Offset     | Field           | Description (numbers are little endian)                                     |
| ---------- | --------------- | --------------------------------------------------------------------------- |
| `[0..8)`   | Magic           | 64-bit magic string: `"DB Pages"` (`0x7365676150204244`).                   |
| `[8..10)`  | File version    | 16-bit unsigned version number. Must be `0`.                                |
| `[10..12)` | Schema version  | 16-bit unsigned version number. Must be `0`.                                |
| `[12..14)` | Page size       | 16-bit unsigned page size. Default is `8192`. A value of `0` means `65536`. |
| `[14..16)` | ---             | ---                                                                         |
| `[16..24)` | Freelist root   | 64-bit unsigned page ID of the first free page.                             |

### Free pages

Each free page contains a single 8-byte header, which is a single 64-bit unsigned page ID of the next free page. The last free page contains a zero pointer.

### Node pages

Each node page contains a 4-byte header.

| Offset     | Field           | Description (numbers are little endian)                                                                                                                                              |
| ---------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `[0..2)`   | Page type       | 16-bit unsigned page type.                                                                                                                                                           |
| `[2..4)`   | Cell count      | 16-bit unsigned cell count.                                                                                                                                                          |
| `[4..N)`   | Tuple data      | Starts with `cell_count` 16-bit unsigned integers pointing to each cell.<br>These pointers are always monotonously decreasing.<br>Cells are tightly packed from the end of the page. |

Currently supported page types:

- `0`: B+ tree internal node page.
- `1`: B+ tree leaf node page.
- `2`: Prolly tree internal node page.
- `3`: Prolly tree leaf node page.

Since cells are always tightly packed, the pointers are not absolutely necessary. More study on their performance impact is needed.

The content of each cell depends on the page type:

- B+ tree internal node pages: each cell except the last one *(the one nearest to the beginning of the page)* contains a `(pointer, key)` pair. The last cell contains `(pointer, height)`. Each `pointer` is an 8-byte unsigned integer denoting some page ID. The `height` of the node is a single byte unsigned integer.
- B+ tree leaf node pages: each cell contains a `(key, value)` pair.

## The WAL file

The WAL file is simply an array of records.

*(WIP)*
