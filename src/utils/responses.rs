use serde::{Serialize, Deserialize};


// JWT RESPONSE STRUCT
#[derive(Serialize,Deserialize)]
pub struct JwtResponse{
  pub access: String,
  pub refresh: String
}

impl JwtResponse {
  pub async fn new(access:String, refresh:String) -> Self {
    Self {
      access,
      refresh
    }
  }
}

// TOKEN RESPONSE STRUCT
#[derive(Serialize,Deserialize,Debug)]
pub struct Token{
  pub token: Option<String>
}

// RETURN MESSAGE STRUCT
#[derive(Serialize, Deserialize)]
pub struct ReturnMessage{
  message:String
}

impl ReturnMessage{
  pub async fn new(message:String) -> Self{
    Self {
      message
    }
  }
}

// RETURN ERROR STRUCT
#[derive(Serialize, Deserialize)]
pub struct ReturnError{
  error:String
}

impl ReturnError{
  pub async fn new(message:String) -> Self{
    Self {
      error: message
    }
  }
}

// RETURN ERRORS STRUCT
#[derive(Serialize, Deserialize)]
pub struct ReturnErrors{
  errors:Vec<String>
}

impl ReturnErrors {
  pub async fn new(errors:Vec<String>) -> Self{
    Self {
      errors
    }
  }
}