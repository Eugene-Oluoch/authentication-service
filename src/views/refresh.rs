use actix_web::{HttpResponse, post, web};
use jwt_simple::prelude::{MACLike, JWTClaims, HS256Key};
use crate::{utils::{responses::{Token, ReturnError}, jwt::{CustomClaim, generate_tokens}}, shared_state::SharedState};



#[post("/refresh")]
pub async fn refresh_tokens(data:web::Json<Token>,state:web::Data<SharedState>) -> HttpResponse {

  let token = data.0;

  // VALIDATE TOKEN IS SUPPLIED
  if token.token.is_none(){
    return HttpResponse::BadRequest().json(ReturnError::new("Refresh token is required.".to_string()).await);
  }

  match state.key.verify_token::<CustomClaim>(&token.token.unwrap(), None){
    Err(_) => HttpResponse::BadRequest().json(ReturnError::new("Token is invalid.".to_string()).await),
    Ok(data) => validate_refresh(data, &state.key).await
  }

}

// VALIDATE REFRESH AND GENERATE TOKENS
async fn validate_refresh(data:JWTClaims<CustomClaim>,key:&HS256Key) -> HttpResponse {
  match data.custom.access {
    true => HttpResponse::BadRequest().json(ReturnError::new("Provide a valid refresh key.".to_string()).await),
    false => {
      let tokens = generate_tokens(key, data.custom).await;
      HttpResponse::Ok().json(tokens)
    }
  }
}