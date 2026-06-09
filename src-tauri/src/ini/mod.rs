pub mod encoding;
pub mod parser;
pub mod paths;
pub mod platform;
pub mod writer;

pub use parser::read_ini_file;
pub use writer::{merge_ini, remove_ini_keys, write_ini_file, write_ini_file_with_encoding_hint};
