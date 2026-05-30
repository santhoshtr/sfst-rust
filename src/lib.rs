use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Opaque handle into the C++ wrapper; only ever used behind a pointer.
#[repr(C)]
struct SfstHandle {
    _private: [u8; 0],
}

#[link(name = "sfst_wrapper", kind = "static")]
unsafe extern "C" {
    fn sfst_init(filename: *const c_char, err: *mut c_int) -> *mut SfstHandle;
    fn sfst_cleanup(handle: *mut SfstHandle);
    fn sfst_analyse(
        handle: *mut SfstHandle,
        input: *const c_char,
        result_count: *mut c_int,
    ) -> *mut *mut c_char;
    fn sfst_generate(
        handle: *mut SfstHandle,
        input: *const c_char,
        result_count: *mut c_int,
    ) -> *mut *mut c_char;
    fn sfst_free_results(results: *mut *mut c_char, count: c_int);
}

#[derive(Debug)]
pub enum SfstError {
    InvalidInput(String),
    FileError(String),
    AllocationError,
}

impl std::fmt::Display for SfstError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SfstError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SfstError::FileError(msg) => write!(f, "File error: {}", msg),
            SfstError::AllocationError => write!(f, "Memory allocation error"),
        }
    }
}

impl std::error::Error for SfstError {}

/// A loaded transducer.
///
/// Each `Sfst` owns its own transducer; instances are independent and dropping
/// one never affects another. The underlying graph is frozen at load time and
/// queries are read-only, so a single `Sfst` is safe to share and query
/// concurrently from multiple threads (e.g. via `Arc<Sfst>`).
pub struct Sfst {
    handle: *mut SfstHandle,
}

// Safe: `handle` points to a transducer that is read-only after construction
// (the binary-file constructor freezes the node index) and the C wrapper holds
// no shared mutable state, so concurrent `&self` queries do not race. See
// upstream commit c5a3fe9 ("Fix non-reentrant analysis").
unsafe impl Send for Sfst {}
unsafe impl Sync for Sfst {}

impl Sfst {
    /// Load a transducer from a file.
    pub fn new(filename: &str) -> Result<Self, SfstError> {
        let c_filename = CString::new(filename)
            .map_err(|_| SfstError::InvalidInput("Filename contains null bytes".to_string()))?;

        let mut err: c_int = 0;
        let handle = unsafe { sfst_init(c_filename.as_ptr(), &mut err) };

        if handle.is_null() {
            return Err(match err {
                1 => SfstError::InvalidInput("Filename is null".to_string()),
                2 => SfstError::FileError("Could not open file".to_string()),
                3 => SfstError::FileError("Error loading transducer".to_string()),
                _ => SfstError::FileError("Unknown error".to_string()),
            });
        }

        Ok(Sfst { handle })
    }

    /// Analyze a string using the loaded transducer.
    pub fn analyse(&self, input: &str) -> Result<Vec<String>, SfstError> {
        self.query(input, sfst_analyse)
    }

    /// Generate a string using the loaded transducer.
    pub fn generate(&self, input: &str) -> Result<Vec<String>, SfstError> {
        self.query(input, sfst_generate)
    }

    fn query(
        &self,
        input: &str,
        call: unsafe extern "C" fn(*mut SfstHandle, *const c_char, *mut c_int) -> *mut *mut c_char,
    ) -> Result<Vec<String>, SfstError> {
        let c_input = CString::new(input)
            .map_err(|_| SfstError::InvalidInput("Input contains null bytes".to_string()))?;

        let mut result_count: c_int = 0;
        let results = unsafe { call(self.handle, c_input.as_ptr(), &mut result_count) };

        if results.is_null() {
            if result_count == 0 {
                return Ok(Vec::new());
            }
            return Err(SfstError::AllocationError);
        }

        let mut rust_results = Vec::with_capacity(result_count as usize);

        for i in 0..result_count {
            let c_str_ptr = unsafe { *results.offset(i as isize) };
            if c_str_ptr.is_null() {
                unsafe { sfst_free_results(results, result_count) };
                return Err(SfstError::AllocationError);
            }

            let c_str = unsafe { CStr::from_ptr(c_str_ptr) };
            rust_results.push(c_str.to_string_lossy().into_owned());
        }

        unsafe { sfst_free_results(results, result_count) };
        Ok(rust_results)
    }
}

impl Drop for Sfst {
    fn drop(&mut self) {
        unsafe { sfst_cleanup(self.handle) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::thread;

    fn get_test_file_path() -> PathBuf {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(manifest_dir).join("test").join("easy.a")
    }

    #[test]
    fn test_analyse_and_generate() {
        let test_file = get_test_file_path();

        let sfst = Sfst::new(test_file.to_str().unwrap()).unwrap();

        let analysis_results = sfst.analyse("easy").unwrap();
        assert_eq!(analysis_results.len(), 1);
        assert_eq!(analysis_results[0], "easy<ADJ><pos>");

        let generate_results = sfst.generate("easy<ADJ><comp>").unwrap();
        assert_eq!(generate_results.len(), 1);
        assert_eq!(generate_results[0], "easier");
    }

    #[test]
    fn test_independent_instances() {
        let test_file = get_test_file_path();
        let path = test_file.to_str().unwrap();

        let a = Sfst::new(path).unwrap();
        let b = Sfst::new(path).unwrap();

        // Dropping one instance must not affect the other.
        drop(a);

        let results = b.analyse("easy").unwrap();
        assert_eq!(results, vec!["easy<ADJ><pos>".to_string()]);
    }

    #[test]
    fn test_concurrent_shared_instance() {
        let test_file = get_test_file_path();
        let sfst = Arc::new(Sfst::new(test_file.to_str().unwrap()).unwrap());

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let sfst = Arc::clone(&sfst);
                thread::spawn(move || {
                    for _ in 0..100 {
                        let results = sfst.analyse("easy").unwrap();
                        assert_eq!(results, vec!["easy<ADJ><pos>".to_string()]);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
