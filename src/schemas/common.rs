use rust_decimal::Decimal;
use serde::Serializer;

pub fn round_currency<S>(x: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.round_dp(2).to_string())
}

pub fn round_currency_option<S>(x: &Option<Decimal>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(d) => round_currency(d, s),
        None => s.serialize_none(),
    }
}
