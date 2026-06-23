mod merge;
mod serialize;

pub use merge::{merge_ini, remove_ini_keys};
pub use serialize::write_ini_file_with_encoding_hint;

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
