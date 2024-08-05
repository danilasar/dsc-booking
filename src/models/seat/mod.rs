pub(crate) mod seat_type;
pub(crate) mod availability_status;

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::PostgresMapper;
use tokio_postgres::{Client, Row};
use tokio_postgres::types::{FromSql, Type};
use crate::core::errors::DbError;
use crate::models::seat::availability_status::AvailabilityStatus;
use crate::models::seat::seat_type::SeatType;


#[derive(Clone, Deserialize, Serialize)]
pub struct Seat {
    pub(crate) id: Option<i32>,
    pub(crate) name: Option<String>,
    pub(crate) seat_type: Option<SeatType>,
    pub(crate) availability: Option<AvailabilityStatus>,
    pub(crate) default_x: Option<f64>,
    pub(crate) default_y: Option<f64>,
    pub(crate) default_rot: Option<f64>,
    pub(crate) x: Option<f64>,
    pub(crate) y: Option<f64>,
    pub(crate) rot: Option<f64>
}

impl std::convert::From<&tokio_postgres::Row> for Seat {
    fn from(row: &Row) -> Self {
        Self {
            id:  row.try_get("id").unwrap_or_else(|e| None),
            name: row.try_get("name").unwrap_or_else(|e| None),
            seat_type: row.try_get("type").unwrap_or_else(|e| None),
            availability: row.try_get("availability").unwrap_or_else(|e| None),
            default_x: row.try_get("default_x").unwrap_or_else(|e| None),
            default_y: row.try_get("default_y").unwrap_or_else(|e| None),
            default_rot: row.try_get("default_rot").unwrap_or_else(|e| None),
            x: row.try_get("x").unwrap_or_else(|e| None),
            y: row.try_get("y").unwrap_or_else(|e| None),
            rot: row.try_get("rot").unwrap_or_else(|e| None)
        }
    }
}


pub(crate) async fn get_all_seats(client: &Client) -> Result<Vec<Seat>, DbError> {
    let stmt = include_str!("../sql/seat/get_all_seats.sql");
    let stmt = client.prepare(&stmt).await?;
    let output = client.query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Seat::from(row))
        .collect::<Vec<Seat>>();
    Ok(output)
}

