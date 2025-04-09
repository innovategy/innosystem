use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

/// Custom wrapper around BigDecimal for Diesel and Serde compatibility
#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
#[diesel(sql_type = Numeric)]
pub struct CustomDecimal(pub BigDecimal);

impl From<BigDecimal> for CustomDecimal {
    fn from(value: BigDecimal) -> Self {
        CustomDecimal(value)
    }
}

impl From<CustomDecimal> for BigDecimal {
    fn from(value: CustomDecimal) -> Self {
        value.0
    }
}

impl FromSql<Numeric, Pg> for CustomDecimal {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = String::from_utf8(bytes.as_bytes().to_vec())?;
        let decimal = BigDecimal::from_str(&value)
            .map_err(|e| format!("Error parsing numeric as BigDecimal: {}", e))?;
        Ok(CustomDecimal(decimal))
    }
}

impl ToSql<Numeric, Pg> for CustomDecimal {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        write!(out, "{}", self.0)?;
        Ok(serialize::IsNull::No)
    }
}
