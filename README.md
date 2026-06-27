# SFST Rust Binding

This is a Rust binding for the SFST (Stuttgart Finite State Transducer Tools) library. It provides a safe, idiomatic Rust interface to the C++ SFST library for finite state transducer operations.

## Features

- **Safe API**: Memory-safe Rust wrapper around the C++ SFST library
- **Two API styles**: Functional API and RAII-style API for automatic resource management
- **Error handling**: Comprehensive error types with descriptive messages
- **Python-compatible**: API mirrors the existing Python binding

## Prerequisites

- Rust toolchain (1.70+)
- C++ compiler (g++ or clang++)
- SFST source files (should be in `../src/` relative to this directory)

## Building

1. Clone or place this `rust/` directory alongside the SFST source code:
   ```
   project/
   ├── src/           # SFST C++ source files
   ├── python/        # Python bindings (optional)
   └── rust/          # This Rust binding
   ```

2. Build the library:
   ```bash
   cd rust
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run the example/test binary:
   ```bash
   cargo run --bin main
   ```

## Usage

```rust
use sfst::Sfst;

// Load a transducer; it is freed automatically when `sfst` is dropped.
let sfst = Sfst::new("path/to/transducer.a")?;

// Analyze and generate
let analysis = sfst.analyse("easier")?;
let generation = sfst.generate("easy<ADJ><comp>")?;
```

Each `Sfst` owns its own transducer and instances are independent. A loaded
transducer is read-only, so a single `Sfst` is `Send + Sync` and can be shared
across threads (e.g. `Arc<Sfst>`) and queried concurrently without locking.

### Error Handling

```rust
use sfst::{Sfst, SfstError};

match Sfst::new("transducer.a") {
    Ok(sfst) => {
        match sfst.analyse("word") {
            Ok(results) => println!("Results: {:?}", results),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Err(SfstError::FileError(msg)) => {
        eprintln!("Could not load transducer: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## API Reference

- `Sfst::new(filename: &str) -> Result<Sfst, SfstError>` - Load a transducer from a file
- `sfst.analyse(input: &str) -> Result<Vec<String>, SfstError>` - Analyze a string
- `sfst.generate(input: &str) -> Result<Vec<String>, SfstError>` - Generate a string

### Error Types

```rust
pub enum SfstError {
    InvalidInput(String),     // Invalid input parameters
    FileError(String),        // File loading/reading errors
    AllocationError,          // Memory allocation failure
}
```

## Testing

The crate includes comprehensive tests that mirror the Python test suite:

```bash
# Run library tests
cargo test

# Run the example binary (requires test files)
cargo run --bin main
```

The tests expect the test file `python/tests/easy.a` to exist. Make sure you have the complete SFST project structure.

## License

This binding follows the same license as the original SFST library.
