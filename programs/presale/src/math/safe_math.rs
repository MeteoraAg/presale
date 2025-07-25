use crate::errors::PresaleError;
use anchor_lang::solana_program::msg;
use std::panic::Location;

pub trait SafeMath<T>: Sized {
    fn safe_add(self, rhs: Self) -> Result<Self, PresaleError>;
    fn safe_mul(self, rhs: Self) -> Result<Self, PresaleError>;
    fn safe_div(self, rhs: Self) -> Result<Self, PresaleError>;
    fn safe_rem(self, rhs: Self) -> Result<Self, PresaleError>;
    fn safe_sub(self, rhs: Self) -> Result<Self, PresaleError>;
    fn safe_shl(self, offset: T) -> Result<Self, PresaleError>;
    fn safe_shr(self, offset: T) -> Result<Self, PresaleError>;
}

macro_rules! checked_impl {
    ($t:ty, $offset:ty) => {
        impl SafeMath<$offset> for $t {
            #[track_caller]
            fn safe_add(self, v: $t) -> Result<$t, PresaleError> {
                match self.checked_add(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_sub(self, v: $t) -> Result<$t, PresaleError> {
                match self.checked_sub(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_mul(self, v: $t) -> Result<$t, PresaleError> {
                match self.checked_mul(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_div(self, v: $t) -> Result<$t, PresaleError> {
                match self.checked_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_rem(self, v: $t) -> Result<$t, PresaleError> {
                match self.checked_rem(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_shl(self, v: $offset) -> Result<$t, PresaleError> {
                match self.checked_shl(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }

            #[track_caller]
            fn safe_shr(self, v: $offset) -> Result<$t, PresaleError> {
                match self.checked_shr(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }
        }
    };
}

checked_impl!(u16, u32);
checked_impl!(i32, u32);
checked_impl!(u32, u32);
checked_impl!(u64, u32);
checked_impl!(i64, u32);
checked_impl!(u128, u32);
checked_impl!(i128, u32);
checked_impl!(usize, u32);

pub trait SafeCast<T>: Sized {
    fn safe_cast(self) -> Result<T, PresaleError>;
}

macro_rules! try_into_impl {
    ($t:ty, $v:ty) => {
        impl SafeCast<$v> for $t {
            #[track_caller]
            fn safe_cast(self) -> Result<$v, PresaleError> {
                match self.try_into() {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(PresaleError::MathOverflow)
                    }
                }
            }
        }
    };
}

try_into_impl!(u128, u64);
