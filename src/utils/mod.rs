pub mod responses;
pub mod jwt;
pub mod enums;
use rand::{thread_rng, Rng, distributions::Alphanumeric};


// TRAITS
pub trait Timestamps {
  fn reset(&mut self);
  fn update(&mut self);
}

// GENERATES A RANDOM CHARACTERS CODE
pub async fn generate_random_code(length:usize) -> String {
    thread_rng()
    .sample_iter(&Alphanumeric)
    .take(length)
    .map(char::from)
    .collect()
}


