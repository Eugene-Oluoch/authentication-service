use serde::{Serialize,Deserialize};
use chrono::Utc;
use uuid::Uuid;

use crate::utils::{enums::{ToUse, OtpChoice}, Timestamps};



#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateUser{
  pub value:String,
  pub auth_identifier:OtpChoice
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User{
  pub _id:Option<String>,
  pub email:Option<String>,
  pub password:Option<String>,
  pub provider:Option<String>,
  pub firstname:Option<String>,
  pub lastname:Option<String>,
  pub auth_identifier:Option<ToUse>,
  pub phone_number:Option<String>,
  pub picture:Option<String>,
  pub verified:Option<bool>,
  pub updated_at: Option<i64>,
  pub created_at: Option<i64>
}

impl Timestamps for User {
  fn reset(&mut self) {
      self._id = Some(Uuid::new_v4().to_string());
      self.updated_at = Some(Utc::now().timestamp_millis());
      self.verified = Some(false);
      self.created_at = Some(Utc::now().timestamp_millis());
  }
  fn update(&mut self) {
      self.updated_at = Some(Utc::now().timestamp_millis());
  }
}
