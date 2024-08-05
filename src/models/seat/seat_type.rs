use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::{FromSql, Type};

#[derive(Clone, Deserialize, Serialize)]
pub(crate) enum SeatType {
    Desk,
    Chair,
    ComputerChair,
    Pouf
}

impl FromSql<'_> for SeatType {
    fn from_sql(
        _sql_type: &Type,
        value: &[u8]
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match value {
            b"desk" => Ok(SeatType::Desk),
            b"chair" => Ok(SeatType::Chair),
            b"computer_chair" => Ok(SeatType::ComputerChair),
            b"pouf" => Ok(SeatType::Pouf),
            _ => Ok(SeatType::Chair),
        }
    }

    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "seat_type"
    }
}

impl Display for SeatType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SeatType::Desk => write!(f, "desk"),
            SeatType::Chair => write!(f, "chair"),
            SeatType::ComputerChair => write!(f, "computer_chair"),
            SeatType::Pouf => write!(f, "pouf"),
        }
    }
}