use crate::bank_account::roles::{ReceiveRole, SenderRole};
use crate::money::{Money, MoneyError};

#[derive(Debug, Clone, Copy)]
pub struct BankAccountId(pub(crate) u32);

#[derive(Debug, Clone, Copy)]
pub struct UserAccountId(pub(crate) u32);

/// 銀行口座(DCIにおけるデータ部分)
#[derive(Debug, Clone)]
pub struct BankAccount {
  id: BankAccountId,
  user_account_id: UserAccountId,
  balance: Money,
}

/// コンテキストに非依存な振る舞い
impl BankAccount {
  pub fn new(id: BankAccountId, user_account_id: UserAccountId, balance: Money) -> Self {
    Self {
      id,
      user_account_id,
      balance,
    }
  }

  pub fn balance(&self) -> &Money {
    &self.balance
  }

  pub fn deposit(mut self, amount: Money) -> Result<BankAccount, MoneyError> {
    self.balance = self.balance.add(amount)?;
    Ok(self)
  }

  pub fn withdraw(mut self, amount: Money) -> Result<BankAccount, MoneyError> {
    self.balance = self.balance.subtract(amount)?;
    Ok(self)
  }
}

/// ロール。
/// 型の定義だけ。いわゆるDCIにおけるメソッドレスロール。
mod roles {
  use crate::bank_account::BankAccount;
  use crate::money::{Money, MoneyError};

  pub trait ReceiveRole {
    fn on_receive(self, money: Money, from: BankAccount) -> Result<Self, MoneyError>
    where
      Self: Sized;
  }

  pub trait SenderRole<T> {
    fn send(self, money: Money, to: T) -> Result<(Self, T), MoneyError>
    where
      Self: Sized;
  }
}

mod role_impl {
  use crate::bank_account::roles::{ReceiveRole, SenderRole};
  use crate::{BankAccount, Money, MoneyError};

  /// 送金先のロールの実装。メソッドフルロール。
  impl ReceiveRole for BankAccount {
    fn on_receive(self, money: Money, _from: BankAccount) -> Result<Self, MoneyError>
    where
      Self: Sized,
    {
      let new_state = self.deposit(money)?;
      Ok(new_state)
    }
  }

  /// 送金元のロールの実装。メソッドフルロール。
  impl<T: ReceiveRole> SenderRole<T> for BankAccount {
    fn send(self, money: Money, to: T) -> Result<(Self, T), MoneyError>
    where
      Self: Sized,
    {
      let new_from = self.withdraw(money.clone())?;
      let new_to = to.on_receive(money, new_from.clone())?;
      Ok((new_from, new_to))
    }
  }
}

/// 送金コンテキスト
/// BankAccountには非依存。送金できるT型として定義する。
mod context {
  use crate::{Money, MoneyError};
  use crate::bank_account::roles::{ReceiveRole, SenderRole};

  pub struct TransferContext<T: ReceiveRole, F: SenderRole<T>> {
    from: F,
    to: T,
  }

  impl<T: ReceiveRole, F: SenderRole<T>> TransferContext<T, F> {
    pub fn new(from: F, to: T) -> Self {
      Self { from, to }
    }
    pub fn transfer(self, money: Money) -> Result<(F, T), MoneyError> {
      self.from.send(money, self.to)
    }
  }
}

#[cfg(test)]
mod tests {
  use iso_4217::CurrencyCode;
  use rust_decimal::Decimal;
  use crate::{BankAccount, BankAccountId, UserAccountId, Money};

  #[test]
  fn test_dci() {
    let ba1 = BankAccount::new(
      BankAccountId(1),
      UserAccountId(1),
      Money::zero(CurrencyCode::JPY),
    );
    let new_ba1 = ba1.deposit(Money::yens_i32(1000)).unwrap();
    let ba2 = BankAccount::new(
      BankAccountId(2),
      UserAccountId(1),
      Money::zero(CurrencyCode::JPY),
    );

    use crate::bank_account::context::TransferContext;
    let context: TransferContext<BankAccount, BankAccount> = TransferContext::new(new_ba1, ba2);
    let (from, to) = context.transfer(Money::yens_i32(10)).unwrap();
    println!("from = {:?}, to = {:?}", from, to);
  }
}
