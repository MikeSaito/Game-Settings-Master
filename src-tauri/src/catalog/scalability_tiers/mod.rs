mod hint;
mod load;
mod types;

pub use hint::tier_hint_for_key;

#[cfg(test)]
#[path = "scalability_tiers_tests.rs"]
mod tests;
