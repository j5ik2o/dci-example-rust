use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

use iso_4217::CurrencyCode;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, Zero};

#[derive(Debug, Clone, PartialEq)]
pub struct Money {
  pub amount: Decimal,
  pub currency: CurrencyCode,
}

#[derive(Debug, PartialEq)]
pub enum MoneyError {
  NotSameCurrencyError,
}

impl Eq for Money {}

impl Hash for Money {
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    self.amount.hash(state);
    state.write_u32(self.currency.num());
  }
}

impl PartialOrd for Money {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self.currency != other.currency {
      None
    } else if self.amount > other.amount {
      Some(Ordering::Greater)
    } else if self.amount < other.amount {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Equal)
    }
  }
}

impl Add for Money {
  type Output = Money;

  fn add(self, rhs: Self) -> Self::Output {
    Money::add(self, rhs).unwrap_or_else(|err| panic!(format!("{:?}", err)))
  }
}

impl Sub for Money {
  type Output = Money;

  fn sub(self, rhs: Self) -> Self::Output {
    Money::subtract(self, rhs).unwrap_or_else(|err| panic!(format!("{:?}", err)))
  }
}

impl Mul<Decimal> for Money {
  type Output = Money;

  fn mul(self, rhs: Decimal) -> Self::Output {
    Money::times(self, rhs)
  }
}

impl Div<Decimal> for Money {
  type Output = Money;

  fn div(self, rhs: Decimal) -> Self::Output {
    Money::divided_by(self, rhs)
  }
}

impl Neg for Money {
  type Output = Money;

  fn neg(self) -> Self::Output {
    Money::negated(self)
  }
}

impl From<(Decimal, CurrencyCode)> for Money {
  fn from((amount, currency): (Decimal, CurrencyCode)) -> Self {
    Money::new(amount, currency)
  }
}

impl From<(&str, CurrencyCode)> for Money {
  fn from((amount, currency): (&str, CurrencyCode)) -> Self {
    let a = Decimal::from_str(amount).unwrap();
    Money::new(a, currency)
  }
}

macro_rules! from_numeric_impl {
  ($($t:ty)*) => ($(
    impl From<($t, CurrencyCode)> for Money {
      fn from((amount, currency): ($t, CurrencyCode)) -> Self {
        let mut a = Decimal::from(amount);
        a.rescale(currency.digit().unwrap() as u32);
        Money::new(a, currency)
      }
    }
  )*)
}

from_numeric_impl! {i8 i16 i32 i64 u8 u16 u32 u64}

impl Money {
  pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
    let mut a = amount;

    a.rescale(currency.digit().unwrap() as u32);
    Self {
      amount: a,
      currency,
    }
  }

  pub fn dollars(amount: Decimal) -> Self {
    Self::new(amount, CurrencyCode::USD)
  }

  pub fn dollars_i32(amount: i32) -> Self {
    Self::dollars(Decimal::from_i32(amount).unwrap())
  }

  pub fn dollars_i64(amount: i64) -> Self {
    Self::dollars(Decimal::from_i64(amount).unwrap())
  }

  pub fn dollars_f32(amount: f32) -> Self {
    Self::dollars(Decimal::from_f32(amount).unwrap())
  }

  pub fn yens(amount: Decimal) -> Self {
    Self::new(amount, CurrencyCode::JPY)
  }

  pub fn yens_i32(amount: i32) -> Self {
    Self::yens(Decimal::from_i32(amount).unwrap())
  }

  pub fn yens_i64(amount: i64) -> Self {
    Self::yens(Decimal::from_i64(amount).unwrap())
  }

  pub fn zero(currency: CurrencyCode) -> Self {
    Self::new(Decimal::zero(), currency)
  }

  pub fn abs(&self) -> Self {
    Self {
      amount: self.amount.abs(),
      currency: self.currency,
    }
  }

  pub fn is_positive(&self) -> bool {
    self.amount > Decimal::zero()
  }

  pub fn is_negative(&self) -> bool {
    self.amount < Decimal::zero()
  }

  pub fn is_zero(&self) -> bool {
    self.amount.is_zero()
  }

  pub fn negated(self) -> Self {
    Self {
      amount: -self.amount,
      currency: self.currency,
    }
  }

  //noinspection RsExternalLinter
  pub fn add(self, other: Self) -> Result<Self, MoneyError> {
    if self.currency != other.currency {
      Err(MoneyError::NotSameCurrencyError)
    } else {
      Ok(Self {
        amount: self.amount + other.amount,
        currency: self.currency,
      })
    }
  }

  pub fn subtract(self, other: Self) -> Result<Self, MoneyError> {
    self.add(other.negated())
  }

  pub fn times(self, factor: Decimal) -> Self {
    Self {
      amount: self.amount * factor,
      currency: self.currency,
    }
  }

  pub fn divided_by(self, divisor: Decimal) -> Self {
    Self {
      amount: self.amount / divisor,
      currency: self.currency,
    }
  }
}

#[cfg(test)]
mod tests {
  use iso_4217::CurrencyCode;
  use rust_decimal::Decimal;
  use crate::money::{Money};
  use rust_decimal::prelude::{Zero, FromPrimitive};

  #[test]
  fn test_eq() {
    let m1 = Money::from((1u32, CurrencyCode::USD));
    let m2 = Money::from((1u32, CurrencyCode::USD));
    assert_eq!(m1, m2);
  }

  #[test]
  fn test_ne() {
    let m1 = Money::from((1u32, CurrencyCode::USD));
    let m2 = Money::from((2u32, CurrencyCode::USD));
    assert_ne!(m1, m2);
  }

  #[test]
  fn test_zero() {
    let m1 = Money::zero(CurrencyCode::USD);
    let m2 = Money::new(Decimal::zero(), CurrencyCode::USD);
    assert_eq!(m1.abs(), m2);
  }

  #[test]
  fn test_abs() {
    let m1 = Money::new(Decimal::from_i32(-1).unwrap(), CurrencyCode::USD);
    let m2 = Money::new(Decimal::from_i32(1).unwrap(), CurrencyCode::USD);
    assert_eq!(m1.abs(), m2);
  }

  #[test]
  fn test_add() {
    let m1 = Money::from((1u32, CurrencyCode::USD));
    let m2 = Money::from((2u32, CurrencyCode::USD));
    let m3 = m1.clone();
    let m4 = m2.clone();

    let m5 = m1.add(m2).unwrap();
    let m6 = m3 + m4;

    assert_eq!(
      m5,
      Money::new(Decimal::from_i32(3).unwrap(), CurrencyCode::USD)
    );
    assert_eq!(
      m6,
      Money::new(Decimal::from_i32(3).unwrap(), CurrencyCode::USD)
    );
  }
}
