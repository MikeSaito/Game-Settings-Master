pub mod encoding;
pub mod parser;
pub mod patch;
pub mod paths;
pub mod platform;
pub mod writer;

pub use parser::read_ini_file;
pub use patch::{
    expand_mirror_key_updates, filter_updates_to_existing_keys, patch_ini_text,
};
pub use writer::{merge_ini, remove_ini_keys, write_ini_file_with_encoding_hint};
