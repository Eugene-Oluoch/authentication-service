use mongodb::Client;
use jwt_simple::prelude::HS256Key;
use regex::Regex;

#[derive(Clone)]
pub struct SharedState{
  pub client:Client,
  pub key:HS256Key,
  pub email_regex:Regex,
  pub phone_regex:Regex
}

impl SharedState{
  pub fn new(client:Client,key:HS256Key) -> Self {
    Self {
      client ,
      key ,
      email_regex: Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap() ,
      phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
    }
  }
}