# Rust Error Handling — From Beginner to Production

> Using `save_to_disk` as a live case study.

---

## Table of Contents

1. [The Problems in the Current Code](#1-the-problems-in-the-current-code)
2. [Core Concepts You Must Internalize](#2-core-concepts-you-must-internalize)
3. [Build a Proper Error Type](#3-build-a-proper-error-type)
4. [Rewrite: save_to_disk the Right Way](#4-rewrite-save_to_disk-the-right-way)
5. [The Full Annotated Diff](#5-the-full-annotated-diff)
6. [Production Patterns Cheat Sheet](#6-production-patterns-cheat-sheet)

---

## 1. The Problems in the Current Code

### 1.1 — `SaveToFileErr` is an information black hole

```rust
// CURRENT — BAD
pub struct SaveToFileErr;
```

This error carries **zero information**. When it reaches the caller, they cannot:
- Know *what* failed (file create? write? path issue?)
- Know *why* it failed (permission denied? disk full? invalid path?)
- Log anything useful
- Recover intelligently

**Rule:** Every error type must carry enough context for the caller to either recover or produce a useful log line. At minimum, wrap the underlying source error.

---

### 1.2 — `std::io::Error` is silently thrown away

```rust
// CURRENT — BAD
Err(err) => Err(SaveToFileErr)   // `err` is ignored — the real cause is gone forever
```

You are wrapping a rich OS error (containing errno, message, etc.) into an empty struct. This is like catching an exception and rethrowing `Exception("something went wrong")`. The original error has to be **carried forward**, not discarded.

---

### 1.3 — `.expect()` inside a function that returns `Result`

```rust
// CURRENT — BAD
let mut file = OpenOptions::new()
    .append(true)
    .open(self.file_name.clone())
    .expect("Cannot open file");   // ← PANIC in library code
```

`.expect()` is a controlled panic. **Panics must never appear in library or reusable code.** If this function returns `Result<_, _>`, every failure path must go through `Err(...)`, never through a panic. The only acceptable use of `.expect()` is in `main()`, tests, or one-off scripts where you are certain the invariant holds and you intentionally want to abort.

**Same issue here:**
```rust
// CURRENT — BAD
let value = self.get(key).expect("key doesnt exists");
```

You are iterating `self.data.keys()` — the key is guaranteed to exist. But the fix isn't `.expect()`, it's to **iterate correctly** so the type system proves it can't fail (see §4).

---

### 1.4 — `file.write(content)` result is silently discarded

```rust
// CURRENT — BAD
file.write(content);   // #[must_use] return value dropped — compiler warned you
```

`Write::write` returns `Result<usize, io::Error>`. Ignoring it means a partial or failed write looks identical to a successful one. The compiler actually emits a **`#[must_use]` warning** here. Treat warnings as errors in production code (`#![deny(warnings)]` or `#![deny(unused_must_use)]`).

---

### 1.5 — TOCTOU race condition

```rust
// CURRENT — BAD
match fs::exists(self.file_name.clone()) {   // CHECK
    Ok(true) => {
        OpenOptions::new().append(true).open(...)   // USE  ← gap here
    }
    Ok(false) => self.create_file(content)
}
```

Between `fs::exists` and `open`, another process can create or delete the file. This is a **Time-Of-Check / Time-Of-Use (TOCTOU)** race. The OS provides atomic primitives specifically to avoid this. `OpenOptions` lets you express "create if not exists, otherwise truncate/append" in a **single syscall**.

---

### 1.6 — Hardcoded path inside `create_file`

```rust
// CURRENT — BAD
fn create_file(&self, content: &[u8]) -> Result<usize, std::io::Error> {
    match File::create_new("store.txt") {   // ← ignores `self.file_name`
```

The struct has a `file_name` field. `create_file` ignores it and hardcodes `"store.txt"`. If you ever change `file_name`, `create_file` breaks silently.

---

### 1.7 — `Ok(true)` / `Ok(false)` is a meaningless distinction

```rust
// CURRENT — BAD
pub fn save_to_disk(&self) -> Result<bool, SaveToFileErr>
```

What does `Ok(false)` mean? "It succeeded but also kinda didn't?" If the operation fails, return `Err`. If it succeeds, return `Ok(())`. Using a boolean inside `Ok` is a code smell that signals unclear semantics.

**Rule:** `Result<(), E>` for operations with no meaningful return value. `Result<T, E>` only when the `T` value is actually used by the caller.

---

### 1.8 — Inefficient string building

```rust
// CURRENT — BAD
let mut content = "".to_string();
for key in self.data.keys() {
    let key_value = format!("{}: {}\n", key, value);
    content = content.to_owned() + &key_value;   // ← allocates a NEW String every iteration
}
```

`to_owned()` on a `String` clones it. Each loop iteration: clone the whole string, allocate `key_value`, concat, drop the old allocation. For N keys, this is O(N²) allocations. Use `push_str` or `String::with_capacity` + `write!`.

---

### 1.9 — `.clone()` where a borrow suffices

```rust
// CURRENT — BAD
fs::exists(self.file_name.clone())
OpenOptions::new().open(self.file_name.clone())
```

`fs::exists` and `open` both take `AsRef<Path>`. `&String` implements `AsRef<Path>`. You never need to clone here — pass `&self.file_name`.

---

### 1.10 — `keys()` has side effects and returns nothing

```rust
// CURRENT — questionable design
pub fn keys(&self) {
    println!("{key}");   // I/O side effect baked into the data layer
}
```

A data store should not decide how its contents are displayed. It should return data; the caller decides how to present it. Mix data and I/O and you make the store untestable and inflexible.

---

## 2. Core Concepts You Must Internalize

### 2.1 — The `?` operator is your best friend

```rust
// VERBOSE — unnecessary
match some_result {
    Ok(val) => val,
    Err(e) => return Err(e),
}

// IDIOMATIC — use ?
let val = some_result?;
```

`?` also calls `.into()` on the error, so it automatically converts error types as long as a `From` impl exists. This is the foundation of ergonomic error propagation.

---

### 2.2 — Implement the error trait trinity

For any error type you publish or use seriously, implement:

| Trait | Why |
|---|---|
| `Debug` | Required by `unwrap()`/`expect()`, test output |
| `Display` | Human-readable message for logs and users |
| `std::error::Error` | Interop with the ecosystem (`Box<dyn Error>`, `anyhow`, etc.) |

```rust
use std::fmt;

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    EmptyStore,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "I/O error: {e}"),
            StoreError::EmptyStore => write!(f, "cannot persist: store is empty"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            StoreError::EmptyStore => None,
        }
    }
}

// Lets you use `?` to convert io::Error → StoreError automatically
impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}
```

---

### 2.3 — `From` enables `?` across error type boundaries

Because of `impl From<std::io::Error> for StoreError`, the `?` operator can now auto-convert:

```rust
fn save(&self) -> Result<(), StoreError> {
    let mut file = File::create("foo.txt")?;   // io::Error → StoreError::Io via From
    file.write_all(b"data")?;                  // same
    Ok(())
}
```

No manual `match`, no information loss, no boilerplate.

---

### 2.4 — Use `write_all`, not `write`

`Write::write` is allowed to write **fewer bytes than requested** (partial write). `Write::write_all` loops until all bytes are written or an error occurs. In almost every application-level use case, you want `write_all`.

```rust
file.write(content)?;      // may write 0..N bytes — BAD
file.write_all(content)?;  // writes all N bytes or errors — GOOD
```

---

### 2.5 — Atomic file writes for durability

For a KV store, the worst failure mode is a corrupted file: write started, process crashed mid-write, file is now half-updated garbage. The production pattern is:

```
write → temp file (same filesystem)
fsync  → temp file
rename → final path   (atomic on POSIX)
```

`rename` is atomic on Linux/macOS. Either the old file or the new file is visible — never a half-written hybrid.

---

## 3. Build a Proper Error Type

Here is a production-grade error type for this store:

```rust
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum StoreError {
    /// Wraps any I/O failure, preserving the original OS error.
    Io(io::Error),
    /// Caller attempted to persist an empty store.
    EmptyStore,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "store I/O error: {e}"),
            StoreError::EmptyStore => write!(f, "cannot save: store contains no entries"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            StoreError::EmptyStore => None,
        }
    }
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}
```

---

## 4. Rewrite: `save_to_disk` the Right Way

### Basic version — correct and idiomatic

```rust
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};

pub fn save_to_disk(&self) -> Result<(), StoreError> {
    if self.data.is_empty() {
        return Err(StoreError::EmptyStore);
    }

    // Build the content string efficiently — no cloning, pre-allocated capacity.
    let mut content = String::with_capacity(self.data.len() * 32);
    for (key, value) in &self.data {
        // write! into a String never fails (infallible), so unwrap is correct here.
        write!(content, "{key}: {value}\n").unwrap();
    }

    // OpenOptions handles "create if missing, truncate if present" atomically —
    // no need to check existence first (eliminates TOCTOU).
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&self.file_name)?;            // io::Error auto-converts via From

    // BufWriter batches small writes into fewer syscalls.
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;  // write_all guarantees full write
    writer.flush()?;                         // flush BufWriter's internal buffer

    Ok(())
}
```

**What changed and why:**

| Before | After | Reason |
|---|---|---|
| `SaveToFileErr` (empty) | `StoreError::Io(io::Error)` | Preserves the real OS error |
| `fs::exists` + `open` | Single `OpenOptions` call | Eliminates TOCTOU race |
| `.expect()` inside fn | `?` operator | Propagates error, no panic |
| `file.write(content)` | `write_all` + `flush` | Guarantees complete write |
| `content.to_owned() + &...` | `String::with_capacity` + `write!` | O(N) instead of O(N²) |
| `self.file_name.clone()` | `&self.file_name` | No unnecessary allocation |
| `Result<bool, _>` | `Result<(), _>` | Clear semantics |

---

### Production version — atomic write with `BufWriter`

```rust
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};

pub fn save_to_disk(&self) -> Result<(), StoreError> {
    if self.data.is_empty() {
        return Err(StoreError::EmptyStore);
    }

    // Write to a temp file first. On the same filesystem as the target so that
    // rename() is guaranteed to be atomic (no cross-device move).
    let tmp_path = format!("{}.tmp", self.file_name);

    {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;

        let mut writer = BufWriter::new(file);

        for (key, value) in &self.data {
            writeln!(writer, "{key}: {value}")?;  // writeln adds \n, ? propagates errors
        }

        writer.flush()?;
        // `file` is dropped here — OS flushes kernel buffers when last fd closes.
        // For truly durable writes, call file.sync_all() before drop.
    }

    // Atomic rename: either the old file or the new file is visible, never a hybrid.
    fs::rename(&tmp_path, &self.file_name)?;

    Ok(())
}
```

---

## 5. The Full Annotated Diff

```
BEFORE                                  AFTER
──────────────────────────────────────────────────────────────────────
pub struct SaveToFileErr;               #[derive(Debug)]
                                        pub enum StoreError {
                                            Io(io::Error),     // ← source preserved
                                            EmptyStore,
                                        }
                                        impl From<io::Error> for StoreError { ... }

Result<bool, SaveToFileErr>             Result<(), StoreError>
                                        // bool inside Ok was meaningless

fs::exists(...)                         OpenOptions::new()
  + open(...)                               .create(true)
                                            .truncate(true)
                                            .open(...)?
                                        // atomic, no TOCTOU

.expect("Cannot open file")             ?   // propagate, never panic

file.write(content)                     writer.write_all(content.as_bytes())?
                                        writer.flush()?

content = content.to_owned() + &kv     writeln!(writer, "{key}: {value}")?
                                        // write directly, no temp allocations

self.file_name.clone()                  &self.file_name
                                        // AsRef<Path> takes a reference
```

---

## 6. Production Patterns Cheat Sheet

```
Scenario                            Pattern
────────────────────────────────────────────────────────────
Single error type in a crate        Custom enum + From impls
Prototyping / bins only             anyhow::Result<T>
Library crate (published)           thiserror derive macros
Need to attach context to errors    .map_err(|e| MyErr::Io { path: path.clone(), source: e })
Never panic in library code         Replace .expect() with ?
Partial writes                      Always use write_all, never write
Durability                          write → fsync → rename
File create-or-truncate             OpenOptions .create(true).truncate(true).write(true)
File append                         OpenOptions .create(true).append(true)
Buffer many small writes            Wrap File in BufWriter
Must-use return values              Compiler warns — treat warnings as errors
```

---

### Quick reference: `OpenOptions` flags

| Goal | Flags to set |
|---|---|
| Create new, fail if exists | `.create_new(true).write(true)` |
| Create or overwrite | `.create(true).write(true).truncate(true)` |
| Create or append | `.create(true).append(true)` |
| Open existing for writing | `.write(true)` |
| Open existing for reading | `.read(true)` (or just `File::open`) |

---

> **One-line rule to remember:**
> If your function returns `Result`, every failure must exit through `Err(...)`. A panic inside a `Result`-returning function is always a bug.
