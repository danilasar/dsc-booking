use std::fmt::{Display, Formatter, write};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use tokio_postgres::types::{FromSql, Type};
use crate::models::seat::seat_type::SeatType;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) enum AvailabilityStatus {
    Unavailable,
    Taken,
    Free
}

impl FromSql<'_> for AvailabilityStatus {
    fn from_sql(
        _sql_type: &Type,
        value: &[u8]
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match value {
            b"unavailable" => Ok(AvailabilityStatus::Unavailable),
            b"free"        => Ok(AvailabilityStatus::Free),
            b"taken"       => Ok(AvailabilityStatus::Taken),
            b"talen"       => Ok(AvailabilityStatus::Taken), // legacy
            _              => Ok(AvailabilityStatus::Unavailable)
        }
    }

    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "availability_status"
    }
}

impl Display for AvailabilityStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AvailabilityStatus::Unavailable => write!(f, "unavailable"),
            AvailabilityStatus::Taken       => write!(f, "taken"),
            AvailabilityStatus::Free        => write!(f, "free")
        }
    }
}