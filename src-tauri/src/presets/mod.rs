#[cfg(test)]
mod apply_dir;
#[cfg(test)]
mod diff;
pub(crate) mod resolve;
#[cfg(test)]
mod validate;

pub use resolve::resolve_apply_resolution;
#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
