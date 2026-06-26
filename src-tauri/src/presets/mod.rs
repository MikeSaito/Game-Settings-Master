mod apply_dir;
mod apply_targets;
mod diff;
mod resolve;
mod validate;

pub use apply_targets::apply_custom_to_targets;
pub use resolve::resolve_apply_resolution;
#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
