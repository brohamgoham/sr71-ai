//! Exact monetary units shared by the prototype and future world parser.
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Mist(u64);

#[derive(Debug, Error)]
#[error("cannot parse SUI amount {input:?}: {reason}; use a nonnegative decimal with up to 9 places")]
pub struct AmountError {
    input: String,
    reason: &'static str,
}

impl Mist {
    pub const ZERO: Self = Self(0);
    pub const PER_SUI: u64 = 1_000_000_000;

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl FromStr for Mist {
    type Err = AmountError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let fail = |reason| AmountError { input: input.to_owned(), reason };
        let (whole, fraction) = input.split_once('.').unwrap_or((input, ""));
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return Err(fail("invalid whole part"));
        }
        if fraction.len() > 9 || !fraction.bytes().all(|b| b.is_ascii_digit())
            || (input.contains('.') && fraction.is_empty())
        {
            return Err(fail("invalid fractional part"));
        }
        let whole: u64 = whole.parse().map_err(|_| fail("amount overflow"))?;
        let mut fractional = 0_u64;
        for index in 0..9 {
            fractional *= 10;
            if let Some(digit) = fraction.as_bytes().get(index) {
                fractional += u64::from(digit - b'0');
            }
        }
        whole.checked_mul(Self::PER_SUI).and_then(|value| value.checked_add(fractional))
            .map(Self).ok_or_else(|| fail("amount overflow"))
    }
}

impl fmt::Display for Mist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fraction = format!("{:09}", self.0 % Self::PER_SUI);
        let fraction = fraction.trim_end_matches('0');
        write!(f, "{}.{}", self.0 / Self::PER_SUI, if fraction.is_empty() { "0" } else { fraction })
    }
}

impl TryFrom<String> for Mist {
    type Error = String;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        input.parse::<u64>().map(Self)
            .map_err(|_| format!("invalid mist {input:?}; use a u64 decimal string"))
    }
}

impl From<Mist> for String {
    fn from(value: Mist) -> Self {
        value.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_amounts_and_bounds() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!("0.1".parse::<Mist>()?.value(), 100_000_000);
        assert_eq!("0.000000001".parse::<Mist>()?.value(), 1);
        assert_eq!("18446744073.709551615".parse::<Mist>()?.value(), u64::MAX);
        for value in ["", "-1", "+1", "1e2", "1.", ".1", "1.0000000000", "1.x",
            "18446744073.709551616", "18446744074", "999999999999999999999"] {
            assert!(value.parse::<Mist>().is_err(), "{value}");
        }
        assert_eq!(Mist::new(100_000_000).to_string(), "0.1");
        assert_eq!(Mist::ZERO.to_string(), "0.0");
        assert_eq!(Mist::new(u64::MAX).checked_add(Mist::new(1)), None);
        assert_eq!(Mist::ZERO.checked_sub(Mist::new(1)), None);
        assert!(Mist::try_from("-1".to_owned()).is_err());
        Ok(())
    }
}
