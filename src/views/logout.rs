use actix_web::{web, post,HttpResponse, HttpRequest};

use crate::{utils::{responses::{ReturnError}, jwt::TokenBlacklist}, models::db::insert_doc, shared_state::SharedState};

#[post("/logout")]
pub async fn logout(data:HttpRequest,state:web::Data<SharedState>) -> HttpResponse{
  let token;

  let authorization = data.headers().get("authorization");
  match authorization {
    None => return HttpResponse::BadRequest().json(ReturnError::new("Access token is required.".to_string()).await),
    Some(data) => {
      let data= data.to_str().unwrap().to_string();
      let data:Vec<&str> = data.split(" ").collect();
      if data.len() < 2{
        return HttpResponse::BadRequest().json(ReturnError::new("Provide a valid authorization bearer.".to_string()).await)
      }
      token = Some(data[1].to_string());
    }
  }

  // VALIDATE THAT TOKEN IS SUPPLIED
  if token.is_none(){
    return HttpResponse::BadRequest().json(ReturnError::new("Access token is required.".to_string()).await);
  }

  let blacklist_token = TokenBlacklist::new(token.unwrap()).await;
  let blacklisted = insert_doc(&state.client, "tokenblacklists", &blacklist_token).await;

  // IF FAILED TO BLACKLIST
  if blacklisted.is_err(){
    return HttpResponse::InternalServerError().finish()
  }


  HttpResponse::Ok().finish()
}