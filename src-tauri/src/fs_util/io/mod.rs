mod atomic;
mod permissions;
mod read;
mod write;

pub use permissions::{clear_readonly, format_io_error};
pub use read::{read_file_bytes, read_utf8_text_file};
pub use write::{write_file_bytes, write_file_bytes_opts};

#[cfg(test)]
pub use read::strip_utf8_bom;
