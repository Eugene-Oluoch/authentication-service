use chrono::Utc;
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use crate::utils::{enums::OtpChoice, generate_random_code};

#[derive(Serialize,Deserialize,Debug)]
pub struct ForgotCode{
  pub _id: Option<String>,
  pub code: String,
  pub value:String,
  pub auth_identifier:OtpChoice,
  pub archive:Option<bool>,
  pub updated_at: Option<i64>,
  pub created_at: Option<i64>
}


impl ForgotCode{
  pub async fn new(value:&str,auth_identifier:&OtpChoice) -> Self{
    Self {
    _id : Some(Uuid::new_v4().to_string()),
    code : generate_random_code(30).await,
    value:value.to_string(),
    auth_identifier:auth_identifier.clone(),
    archive:Some(false),
    created_at:Some(Utc::now().timestamp_millis()),
    updated_at:Some(Utc::now().timestamp_millis())
    }
    
  }

}