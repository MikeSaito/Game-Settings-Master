mod constants;
mod detect;
mod parse;
mod types;

pub use constants::is_scalability_quality_index;
pub use detect::detect_scalability_limits;
pub use types::ScalabilityLimits;

#[cfg(test)]
#[path = "scalability_tests.rs"]
mod tests;
