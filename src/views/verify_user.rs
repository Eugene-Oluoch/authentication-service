use actix_web::{web, get, HttpResponse};
use mongodb::{bson::{doc, Document}, Client};

use crate::{shared_state::SharedState, models::{db::{get_one, update_one}, otps::Otp, users::User}, utils::{responses::{ReturnError, ReturnMessage}, enums::OtpChoice}};



async fn update_user_otp(client:&Client,user_id:&str,otp_id:&str) -> HttpResponse{
  match update_one::<User>(client, "users", doc!{"$set":{"verified":Some(true)}}, doc!{"_id":user_id}).await{
    Err(err) => return HttpResponse::InternalServerError().json(ReturnError::new(err.to_string()).await),
    Ok(_) => {
      let _ = update_one::<Otp>(client, "otps", doc!{"$set":{"archive":true}}, doc!{"_id":otp_id}).await;
    }
  }
  return HttpResponse::Ok().json(ReturnMessage::new(String::from("Account verified successfully!")).await)
}

async fn user_verified_check(document:Document,otp:Otp,client:&Client)-> HttpResponse{
  let results = get_one::<User>(client, "users",document).await.expect("msg").unwrap();
  match results.verified{
    Some(res) => {
      match res {
        false=> {
          return update_user_otp(client, results._id.as_ref().unwrap(), otp._id.as_ref().unwrap()).await
        },
        true => {
          return HttpResponse::BadRequest().json(ReturnError::new("Account already verified!".to_string()).await)
        }
      }
    },
    None => {
      return update_user_otp(client, results._id.as_ref().unwrap(), otp._id.as_ref().unwrap()).await
    }
  }
}

async fn otp_found_flow(otp:Otp,client:&Client) -> HttpResponse{
  let document = match otp.auth_identifier{
    Some(OtpChoice::PhoneNumber) => doc! {"phone_number":&otp.phone_number,"auth_identifier":"PhoneNumber"},
    None | Some(OtpChoice::Email)=> doc! {"email":&otp.email,"auth_identifier":"Email"}
  };
  user_verified_check(document, otp, client).await
}


#[get("/verify-user/{code}")]
pub async fn verify_user_link(path: web::Path<String>,state:web::Data<SharedState>) -> HttpResponse{
  let code = path.into_inner();
  match get_one::<Otp>(&state.client, "otps", doc!{"code":code}).await{
    Err(err) => HttpResponse::InternalServerError().json(ReturnError::new(err.to_string()).await),
    Ok(data) => {
      match data{
        None => HttpResponse::BadRequest().json(ReturnError::new("Invalid Link".to_string()).await),
        Some(otp) =>{
          otp_found_flow(otp, &state.client).await
        }
      }
    }
  }
}