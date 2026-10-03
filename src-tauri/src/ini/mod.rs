pub mod document;
pub mod encoding;
pub mod parser;
pub mod patch;
pub mod paths;
pub mod platform;
#[cfg(test)]
pub mod writer;

pub use parser::read_ini_file;
pub use patch::expand_mirror_key_updates;
#[cfg(test)]
pub use patch::patch_ini_text;
#[cfg(test)]
pub use writer::{merge_ini, remove_ini_keys, write_ini_file_with_encoding_hint};
