use std::hash::{BuildHasher, Hasher};
use actix_session::{Session, SessionGetError};
use deadpool_postgres::Client;
use rs_sha512::{HasherContext, Sha512State};
use crate::core::errors::DbError;
use crate::models::user::{get_user_by_token, User};

enum GetCurrentUserError {
    SessionGet(SessionGetError), Db(DbError), SessionIsNotString
}

pub(crate) fn hash_password(password:&str, login: &str) -> String {
    let mut sha512hasher = Sha512State::default().build_hasher();
    sha512hasher.write(password.as_bytes());
    sha512hasher.write(format!("СВО{}aboba_AntiHohol",
                               login.clone()).as_bytes());
    let bytes_result = HasherContext::finish(&mut sha512hasher);
    return format!("{bytes_result:02x}");
}



pub async fn get_current_user(client: &Client,
                              session: Session)
                              -> Result<User, GetCurrentUserError>
{
    let token : String = match session.get("token") {
        Ok(token_option) => match token_option {
            Some(val) =>  val,
            None => return Err(GetCurrentUserError::SessionIsNotString)
        },
        Err(error) => return Err(GetCurrentUserError::SessionGet(error))
    };

    match get_user_by_token(&client, token.as_str()).await {
        Ok(user) => Ok(user),
        Err(error) => Err(GetCurrentUserError::Db(error))
    }
}

pub async fn is_authored(client: &Client, session: Session) -> bool {
    return get_current_user(&client, session).await.is_ok();
}