use serde::{Serialize,Deserialize};

// JWT TOKENS ENUM
pub enum Tokens {
  Access,
  Refresh
}

// OTPCHOICE ENUM
#[derive(Clone,Serialize,Deserialize,Debug)]
pub enum OtpChoice {
  Email,
  PhoneNumber
}

// CHOICE TO USE
#[derive(Serialize, Deserialize, Debug,PartialEq)]
pub enum ToUse{
  PhoneNumber,
  Email,
  Provider
}