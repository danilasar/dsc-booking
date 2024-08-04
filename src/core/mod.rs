use actix_session::Session;
use actix_web::{HttpRequest, web};
use deadpool_postgres::Client;
use crate::AppState;
use crate::core::errors::DbError;

pub mod errors;
pub mod db;
pub mod templator;
pub mod users;

pub(crate) struct ServiceData<'a> {
    pub(crate) req: HttpRequest,
    pub(crate) app_state: web::Data<AppState<'a>>,
    pub(crate) session: Session,
    pub(crate) client: Client
}

impl ServiceData<'_> {
    pub(crate) async fn new(req: HttpRequest, app_state: web::Data<AppState<'_>>, session: Session) -> Result<ServiceData, DbError> {
        let client = app_state.db_pool.get().await.map_err(DbError::PoolError)?;
        let data = ServiceData {
            req,
            app_state,
            session,
            client
        };
        Ok(data)
    }
}