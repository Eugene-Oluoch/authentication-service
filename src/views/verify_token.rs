// TODO GRPC ENDPOINT
use std::sync::Arc;
use jwt_simple::prelude::{MACLike, HS256Key, JWTClaims};
use mongodb::{bson::doc, Client};
use crate::{utils::{jwt::{CustomClaim, TokenBlacklist}},models::{db::get_one}, shared_state::SharedState};
use tonic::{Request, Response, Status};
use verify::verify_token_server::{VerifyToken};
use verify::{TokenReceive, TokenResponse};

pub mod verify {
  tonic::include_proto!("verify");
}


#[derive(Debug)]
pub struct TokenService {
  client:Client,
  key:HS256Key
}

impl TokenService{
  pub fn new(state:SharedState) -> Self{
    Self{
      client: state.client,
      key:state.key
    }
  }
}




async fn validate_token_blacklist(client:&Client,token:&Arc<String>) -> Result<Response<TokenResponse>, Status>{
  match get_one::<TokenBlacklist>(client, "tokenblacklists", doc!{"token":Arc::clone(token).to_string()}).await{
    Err(err) => Err(Status::new(tonic::Code::Cancelled, err.to_string())),
    Ok(value) => {
      match value{
        None => Ok(Response::new(TokenResponse { message: "Valid".to_string() })),
        Some(_) => Err(Status::new(tonic::Code::Unknown, "Invalid token"))
      }
    }
  }
}

async fn validate_token_access(client:&Client,token:&Arc<String>,data:JWTClaims<CustomClaim>) -> Result<Response<TokenResponse>, Status> {
  match  data.custom.access{
    false => Err(Status::new(tonic::Code::InvalidArgument, "Invalid token")),
    true => {
      validate_token_blacklist(client, token).await
    }
  }
}


async fn validate_token(client:&Client,key:&HS256Key,token:&Arc<String>) -> Result<Response<TokenResponse>, Status>{
  match key.verify_token::<CustomClaim>(Arc::clone(token).as_str(), None){
    Err(err) => Err(Status::new(tonic::Code::Unavailable, err.to_string())),
    Ok(data) => {
      validate_token_access(client, token, data).await
    }
  }
}

#[tonic::async_trait]
impl VerifyToken for TokenService {
  async fn check(&self, request: Request<TokenReceive>) -> Result<Response<TokenResponse>, Status> {
    let token = Arc::new(request.into_inner().token);
    validate_token(&self.client, &self.key, &token).await
  }
}


