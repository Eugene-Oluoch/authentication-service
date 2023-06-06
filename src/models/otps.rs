use chrono::Utc;
use uuid::Uuid;
use serde::{Serialize,Deserialize};
use crate::utils::enums::OtpChoice;
use crate::utils::{generate_random_code};

#[derive(Serialize, Deserialize,Debug)]
pub struct Otp{
  pub _id:Option<String>,
  pub code:String,
  pub email:Option<String>,
  pub phone_number:Option<String>,
  pub auth_identifier:Option<OtpChoice>,
  pub archive:Option<bool>,
  pub created_at: Option<i64>
}

impl Otp{
  pub async fn new(email:&Option<String>,phone_number:&Option<String>) -> Self{
    Self {
    _id : Some(Uuid::new_v4().to_string()),
    code : generate_random_code(30).await,
    email:email.clone(),
    auth_identifier:Some(OtpChoice::Email),
    phone_number:phone_number.clone(),
    archive:Some(false),
    created_at:Some(Utc::now().timestamp_millis())
    }
    
  }
}
