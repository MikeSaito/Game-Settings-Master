mod mirror;

#[cfg(test)]
mod text;

pub use mirror::expand_mirror_key_updates;
#[cfg(test)]
pub use text::patch_ini_text;

#[cfg(test)]
#[path = "patch_tests.rs"]
mod tests;
