use std::{sync::Arc, option};

use actix_web::{web::{self, Json}, HttpResponse, post};
use jwt_simple::reexports::serde_json;
use mongodb::{bson::{Document, doc}, Client};
use serde::Deserialize;
use tokio::sync::Mutex;
use crate::{utils::{enums::{OtpChoice, ToUse}, responses::{ReturnError, ReturnErrors}, jwt::{generate_tokens, CustomClaim}, Timestamps}, models::{users::User, db::{get_one, insert_doc, update_one}, otps::Otp}, shared_state::SharedState};
use uuid::Uuid;
use base64::decode;
use dotenv::dotenv;
use std::{env::var};


#[derive(Debug, Deserialize)]
struct Claims {
  aud: String,
  email: String,
  email_verified: bool,
  picture: String,
  name: String
}




#[post("/login")]
pub async fn login(data:web::Json<User>,state:web::Data<SharedState>) -> HttpResponse {
    let mut user = data.0;
    user.reset();
    match &user.auth_identifier{
      None => HttpResponse::BadRequest().json(ReturnError::new("Auth identifier is required!".to_string()).await),
      Some(identifier) => {
        match identifier{
          ToUse::Email =>{
            email_phonenumber_route(user,String::from("Email is required!"),&OtpChoice::Email,&state).await
          },
          ToUse::PhoneNumber => {
            email_phonenumber_route(user,String::from("Phone number is required!"),&OtpChoice::PhoneNumber,&state).await
          },
          ToUse::Provider =>{
            let parts: Vec<&str> = user.provider.as_ref().unwrap().split('.').collect();
            if parts.len() != 3{
              HttpResponse::BadRequest().json(ReturnError::new("Invalid JWT format".to_string()).await)
            } else {
              let user_google: Claims = provider_get_user_info(parts).await;

              // GOOGLE CLIENT ID
              if let Err(err) = provider_validate_token(&user_google).await{
                return HttpResponse::BadRequest().json(err)
              }
              let names:Vec<&str> = user_google.name.split(' ').collect(); 
              if names.len() == 2{
                user.firstname = Some(names.get(0).unwrap().to_string());
                user.lastname = Some(names.get(1).unwrap().to_string());
              }else if names.len() == 1{
                user.firstname = Some(names.get(0).unwrap().to_string())
              }

              user.email = Some(user_google.email);
              user.auth_identifier = Some(ToUse::Provider);
              user.picture = Some(user_google.picture);
              user.verified = Some(user_google.email_verified);
              user.auth_identifier = Some(ToUse::Provider);
              // CHECK IF USER EXISTS
              match get_one::<User>(&state.client, "users", doc!{"email":&user.email,"auth_identifier":"Provider"}).await{
                Err(err) => return HttpResponse::InternalServerError().json(ReturnError::new(err.to_string()).await),
                Ok(data) =>{
                  // TODO PASS THE USER ID
                  match data {
                    None => {
                      let user_details = insert_doc(&state.client, "users", &user).await.expect("Failed to destructure user details");
                      let tokens = generate_tokens(&state.key, CustomClaim::new(user).await).await;
                      HttpResponse::Ok().json(tokens)
                    },
                    Some(us) => {
                      // FORMAT THE ID
                      let tokens = generate_tokens(&state.key, CustomClaim::new(user).await).await;
                      HttpResponse::Ok().json(tokens)
                    }
                  }
                }
              }
            }      
          }
        }
      }
    }
}

async fn email_phonenumber_route(user:User,error_message:String,choice:&OtpChoice,state:&web::Data<SharedState>) -> HttpResponse {
  let option;
  match choice {
    OtpChoice::Email => option = &user.email,
    OtpChoice::PhoneNumber => option = &user.phone_number
  }
  match option {
    None => HttpResponse::BadRequest().json(ReturnError::new(error_message).await),
    Some(_) => {
      user_journey(state,user, choice).await
    }
  }
}


pub async fn generate_magic_link(client:&Client,identifier:&OtpChoice,value:&str,user_exists:bool){
  // CREATE A TOKEN TO BE USED FOR MAGIC LINK
  let mut otp = Otp::new(&None,&None).await;

  let doc_for_otp;
  // ADD USER IF HE DOESN'T EXIST
  match identifier {
    OtpChoice::Email => {
      doc_for_otp = doc!{"email":&value};
      if user_exists == false{
        let _ = add_user(client, &OtpChoice::Email, value).await;
      }
    },
    OtpChoice::PhoneNumber => {
      otp.auth_identifier = Some(OtpChoice::PhoneNumber);
      doc_for_otp = doc!{"phone_number":&value};
      if user_exists == false{
        let _ = add_user(client, &OtpChoice::PhoneNumber, value).await;
      }
    }
  }

  match get_one::<Otp>(client, "otps", doc_for_otp).await{
    Err(err) => {
      println!("{}",err.to_string());
    },
    Ok(result) => {
      let mut code = otp.code.clone();
      match result{
        None => {
          match identifier{
            OtpChoice::Email => {
              otp.email = Some(value.to_string());
            },
            OtpChoice::PhoneNumber => {
              otp.phone_number = Some(value.to_string());
            }
          }

          let _ = insert_doc(client, "otps", &otp).await;
        },
        Some(data) => {
          code = data.code;
          match identifier{
            OtpChoice::Email => {
              otp.email = data.email
            },
            OtpChoice::PhoneNumber => {
              otp.phone_number = data.phone_number
            }
          }
        }
      }
      // TODO SEND THE LINK TO EITHER SMS OR EMAIL
      println!("http://localhost:8050/verify-user/{}",code);

    }
  }
}

// USER JOURNEY FOR PHONENUMBER AND EMAIL ROUTE
pub async fn user_journey_document(user:&User,choice:&OtpChoice) -> Document {
  match choice{
    OtpChoice::Email => doc! {"email":&user.email,"auth_identifier":"Email"},
    OtpChoice::PhoneNumber => doc! {"phone_number":&user.phone_number,"auth_identifier":"PhoneNumber"}
  }
}

pub async fn user_journey_get_user(client:&SharedState,user:User,document:Document,choice:&OtpChoice) -> Result<HttpResponse,HttpResponse>{
  match get_one::<User>(&client.client, "users", document).await{
    Err(err) => Err(HttpResponse::InternalServerError().json(ReturnError::new(err.to_string()).await)),
    Ok(value) => {
      match value {
        None => {
          match choice {
            OtpChoice::Email => {
              if client.email_regex.is_match(&user.email.clone().unwrap()) == false{
                return Err(HttpResponse::BadRequest().json(ReturnError::new(String::from("Provide a valid email address.")).await))
              }
              generate_magic_link(&client.client, &OtpChoice::Email, user.email.as_ref().unwrap(),false).await
            },
            OtpChoice::PhoneNumber => {
              if client.phone_regex.is_match(&user.phone_number.clone().unwrap()) == false {
                return Err(HttpResponse::BadRequest().json(ReturnError::new(String::from("Provide a valid phone number.")).await))
              }
              generate_magic_link(&client.client, &OtpChoice::PhoneNumber, user.phone_number.as_ref().unwrap(),false).await
            }
          }
          Err(HttpResponse::Ok().json(ReturnError::new(String::from("Magic link sent")).await))
        },
        Some(mut data) => {
          if data.password.is_none(){
            match data.verified{
              None => Err(HttpResponse::NotAcceptable().json(ReturnError::new(String::from("User isn't verified. Magic Link sent.")).await)),
              Some(verified) => {
                if verified{
                  // CHECK IS FIRST NAME , LAST NAME and PASSWORD ARE PROVIDED AND SAVE HIS ASS AND RETURN TOKENS
                  let (errors,user_thread) = (Arc::new(Mutex::new(Vec::new())),Arc::new(user));
                  let (errors_clone,errors_clone2,errors_clone3) = (Arc::clone(&errors),Arc::clone(&errors),Arc::clone(&errors));
                  let (user_thread_clone,user_thread_clone2,user_thread_clone3) = (Arc::clone(&user_thread),Arc::clone(&user_thread),Arc::clone(&user_thread));

                  let check_first_name = tokio::spawn(async move{
                    if user_thread_clone.firstname.is_none(){
                      errors_clone.lock().await.push(String::from("First name is required!"));
                    }
                  });

                  let check_last_name = tokio::spawn(async move{
                    if user_thread_clone2.lastname.is_none(){
                      errors_clone2.lock().await.push(String::from("Last name is required!"));
                    }
                  });

                  let check_password = tokio::spawn(async move{
                    if user_thread_clone3.password.is_none(){
                      errors_clone3.lock().await.push(String::from("Password is required!"));
                    }
                  });
  
                  let _ = tokio::join!(check_first_name,check_last_name,check_password);
  
                  if errors.lock().await.len() > 0{
                    return Err(HttpResponse::InternalServerError().json(ReturnErrors::new(errors.lock().await.to_vec()).await))
                  }else{
                    data.password = Some(bcrypt::hash(user_thread.password.as_ref().unwrap(), 7).expect("Failed to hash"));
                    // UPDATE USER INFO
                    println!("{:?}",update_one::<User>(&client.client, "users", doc! {"$set":{"firstname":&user_thread.firstname,"lastname":&user_thread.lastname,"password":&data.password}}, doc!{"_id":data._id.as_ref().unwrap()}).await);
  
                    let tokens = generate_tokens(&client.key, CustomClaim::new(data).await).await;
                    Ok(HttpResponse::Ok().json(tokens))
                  }
  
                }else{
                  // SEND MAGIC LINK
                  match choice{
                    OtpChoice::Email => {
                      generate_magic_link(&client.client, &OtpChoice::Email, user.email.as_ref().unwrap(),true).await;
                    },
                    OtpChoice::PhoneNumber =>{ 
                      generate_magic_link(&client.client, &OtpChoice::PhoneNumber, user.phone_number.as_ref().unwrap(),true).await;
                    }
                  }
                  Err(HttpResponse::Ok().json(ReturnError::new(String::from("User isn't verified. Magic Link sent.")).await))
                }
              }
            }
          }else{
            if user.password.is_none(){
              return Err(HttpResponse::BadRequest().json(ReturnError::new(String::from("Password is required")).await))
            }else{
              if bcrypt::verify(user.password.as_ref().unwrap(), &data.password.clone().unwrap()).expect("msg"){
                let tokens = generate_tokens(&client.key, CustomClaim::new(data).await).await;
                Ok(HttpResponse::Ok().json(tokens))
              }else{
                  Err(HttpResponse::BadRequest().json(ReturnError::new("Invalid password. Make sure default is set to the intended value.".to_string()).await))
              }
            }
          }
        }
      }
    }
  }
}





pub async fn user_journey(client:&SharedState,user:User,choice:&OtpChoice) -> HttpResponse {

  


  let document = user_journey_document(&user, choice).await;
  match user_journey_get_user(client, user, document, choice).await{
    Err(err) => err,
    Ok(some) => some
  }
}

pub async fn add_user(client:&Client,choice:&OtpChoice,value:&str) -> Result<bool, String>{

  let mut document = match choice{
    OtpChoice::Email => doc!{"email":value,"auth_identifier":"Email"},
    OtpChoice::PhoneNumber => doc!{"phone_number":value,"auth_identifier":"PhoneNumber"}
  };


  let query_document = match choice{
    OtpChoice::Email => doc!{"email":value},
    OtpChoice::PhoneNumber => doc!{"phone_number":value}
  };

  match get_one::<User>(client, "users", query_document).await{
    Err(err) => Err(err.to_string()),
    Ok(results) => {
      match results{
        None => {
          document.insert("_id", Some(Uuid::new_v4().to_string()));
          document.insert("verified", Some(false));
          match insert_doc(client, "users", &document).await{
            Err(err) => Err(err.to_string()),
            Ok(_) => Ok(true)
          }
        },
        Some(_) => {
          match choice{
            OtpChoice::Email => Err("Provided email address already in use.".to_string()),
            OtpChoice::PhoneNumber => Err("Provided phone number already in use.".to_string()),
          }
        }
      }
    }
  }

}

// PROVIDER FUNCTIONS
async fn provider_get_user_info(parts:Vec<&str>) -> Claims{
  let payload = parts[1];
  let decoded_payload = decode(payload).unwrap();
  let payload_str = String::from_utf8(decoded_payload).unwrap();
  
  serde_json::from_str(&payload_str).unwrap()
}

async fn provider_validate_token(user_info:&Claims) -> Result<(),Json<ReturnError>>{
  // LOADS ENVS
  dotenv().ok();
  // GOOGLE CLIENT ID
  let google_client_id = var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set.");
  if user_info.aud != google_client_id{
    Err(Json(ReturnError::new(String::from("Supplied token is invalid.")).await))
  }else{
    Ok(())
  }
}