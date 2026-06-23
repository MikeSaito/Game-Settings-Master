mod mirror;
mod sections;
mod text;

pub use mirror::expand_mirror_key_updates;
pub use text::patch_ini_text;

#[cfg(test)]
#[path = "patch_tests.rs"]
mod tests;
