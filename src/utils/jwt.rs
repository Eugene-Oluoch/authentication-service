use jwt_simple::{prelude::{HS256Key, Duration, Claims,MACLike, JWTClaims},Error};
use serde::{Serialize, Deserialize};
use crate::models::users::User;

use super::{enums::Tokens, responses::JwtResponse};
use uuid::Uuid;

// CUSTOM CLAIMS
#[derive(Serialize,Deserialize,Clone)]
pub struct CustomClaim{
  pub email: Option<String>,
  pub phone_number: Option<String>,
  pub user_id: Option<String>,
  pub access: bool,
  pub firstname: Option<String>,
  pub lastname: Option<String>
}

impl CustomClaim {
  pub async fn new(user:User) -> Self {
    Self{
      email:user.email,
      phone_number: user.phone_number,
      user_id: user._id,
      access:false,
      firstname:user.firstname,
      lastname:user.lastname
    }
  }
}

// CLAIM FOR AUTH PROVIDERS
pub struct ProviderClaim {
  aud: String,
  email: String,
  email_verified: bool,
  picture: String,
  name: String
}


// TOKEN BLACKLIST
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenBlacklist{
  _id:String,
  token:String
}


impl  TokenBlacklist {
  pub async fn new(token:String) -> Self{
    Self{
      _id: Uuid::new_v4().to_string(),
      token
    }
  }
}

// GENERATES THE SIGNING KEY
pub async fn generate_key() -> HS256Key {
  HS256Key::generate()
}

// SET DURATION AND ACCESS
async fn get_duration_token(choice:&Tokens) -> (Duration,bool){
  match choice {
    Tokens::Access => (Duration::from_mins(10),true),
    Tokens::Refresh => (Duration::from_days(7),false)
  }
}




// VALIDATE A SUPPLIED TOKEN
pub async fn validate_token(token:&str,key:&HS256Key) -> Result<JWTClaims<CustomClaim>,Error> {
  key.verify_token::<CustomClaim>(token, None)
}

// VALIATE A REFRESH TOKE
pub async fn validate_refresh_token(token:&str,key:&HS256Key) -> Result<(), String> {
  match validate_token(token, key).await {
    Err(err) => Err(err.to_string()),
    Ok(details) => {
      access_check(details).await
    }
  }
}

// CHECK IF TOKEN ACCESS
pub async fn access_check(details:JWTClaims<CustomClaim>) -> Result<(), String> {
  match details.custom.access {
    true => Err(String::from("Token is not a refresh token")),
    false => Ok(())
  }
}

// GENERATE TOKEN EITHER ACCESS OR REFRESH
pub async fn generate_token(mut user_claim:CustomClaim,tokens:&Tokens,key:&HS256Key) -> String {
  let (time,access) = get_duration_token(tokens).await;

  // SET TOKEN IF ACCESS OR NOT
  user_claim.access = access;

  let claim = Claims::with_custom_claims(user_claim, time);
  key.authenticate(claim).expect("Failed to create token.")

}


// GENERATE ALL TOKENS AT A GO
pub async fn generate_tokens(key:&HS256Key,user:CustomClaim) -> JwtResponse {
  // TODO ELIMINATE CLONE AND MOVE TO A SHARED REFERENCE
  let user_two = user.clone();
  let (access,refresh) = (generate_token(user, &Tokens::Access, key).await,generate_token(user_two, &Tokens::Refresh, key).await);
  JwtResponse::new(access, refresh).await
}