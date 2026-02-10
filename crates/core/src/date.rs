// SPDX-License-Identifier: MIT

//!
//! The OpenTimeline date type
//!

use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

/// The minimum year allowed in the OpenTimeline system
pub const MIN_YEAR: i64 = -50000;

/// The maximum year allowed in the OpenTimeline system
pub const MAX_YEAR: i64 = 10000;

/// Errors that can arise in relation to a [`Date`]
#[derive(Error, Debug, Clone)]
pub enum DateError {
    /// The day number is not allowed (must be 1 <= day <= 31)
    #[error("Day `{0}` is not allowed")]
    InvalidDay(i64),

    /// The month number is not allowed (must be 1 <= day <= 12)
    #[error("Month `{0}` is not allowed")]
    InvalidMonth(i64),

    /// The day number is not allowed (must be [`MIN_YEAR`] <= day <= [`MAX_YEAR`])
    #[error("Month `{0}` is not allowed")]
    InvalidYear(i64),

    /// Invalid field initialisation.  e.g. the day has been set without the
    /// month also being set
    #[error("e.g. can't set day without setting month")]
    InvalidFields,
}

/// The OpenTimeline date type
///
/// The year field must be set but the day and month fields are optional.  If
/// the day field is set the month field must be set, but if the month field is
/// set, the day field is optional.
#[derive(Serialize, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Date {
    day: Option<Day>,
    month: Option<Month>,
    year: Year,
}

/// The OpenTimeline day type
#[rustfmt::skip]
#[derive(derive_more::Display, Serialize, Eq, PartialEq, Clone, Copy, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Day(u8);

/// The OpenTimeline month type
#[rustfmt::skip]
#[derive(derive_more::Display, Serialize, Eq, PartialEq, Clone, Copy, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Month(u8);

/// The OpenTimeline year type
/// 
/// The minimum year allowed is [`MIN_YEAR`].  The maximum year allowed is
/// [`MAX_YEAR`]
#[rustfmt::skip]
#[derive(derive_more::Display, Serialize, Eq, PartialEq, Clone, Copy, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Year(i32);

impl Day {
    pub fn value(&self) -> u8 {
        self.0
    }

    // TODO
    // pub fn current() -> Self {
    //     Day(0)
    // }
}

impl Month {
    pub fn value(&self) -> u8 {
        self.0
    }

    // TODO
    // pub fn current() -> Self {
    //     Month(0)
    // }
}

impl Year {
    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn min() -> Self {
        Year(MIN_YEAR as i32)
    }

    pub fn max() -> Self {
        Year(MAX_YEAR as i32)
    }

    // TODO
    pub fn current() -> Self {
        Year(2026)
    }
}

impl TryFrom<i64> for Day {
    type Error = DateError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if (1..=31).contains(&value) {
            Ok(Day(value as u8))
        } else {
            Err(DateError::InvalidDay(value))
        }
    }
}

impl TryFrom<i64> for Month {
    type Error = DateError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if (1..=12).contains(&value) {
            Ok(Month(value as u8))
        } else {
            Err(DateError::InvalidMonth(value))
        }
    }
}

impl TryFrom<i64> for Year {
    type Error = DateError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if (MIN_YEAR..=MAX_YEAR).contains(&value) {
            Ok(Year(value as i32))
        } else {
            Err(DateError::InvalidYear(value))
        }
    }
}

// TODO: add visitor so that can deserialise from strings as well?
impl<'de> Deserialize<'de> for Day {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        Day::try_from(value).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

// TODO: add visitor so that can deserialise from strings as well?
impl<'de> Deserialize<'de> for Month {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        Month::try_from(value).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

// TODO: add visitor so that can deserialise from strings as well?
impl<'de> Deserialize<'de> for Year {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        Year::try_from(value).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}

impl Date {
    // TODO: do this properly
    /// Today's date
    pub fn today() -> Self {
        Self::from(None, None, 2026).unwrap()
    }

    /// Create a new [`Date`] if the result will be valid
    pub fn from(day: Option<i64>, month: Option<i64>, year: i64) -> Result<Date, DateError> {
        let mut date = Date {
            day: None,
            month: None,
            year: Year(0),
        };
        date.set_year(year)?;
        date.set_month(month)?;
        date.set_day(day)?;
        Ok(date)
    }

    /// e.g. 1st Jan 2025 format
    pub fn as_long_date_format(&self) -> String {
        // Day
        let day = match self.day() {
            Some(day) => format!("{day}"),
            None => String::new(),
        };

        // Month
        let month = match self.month() {
            Some(month) => match month.value() {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "May",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => panic!("Month value must be 1 <= x <= 12"),
            },
            None => "",
        };

        // Year
        let year = self.year();

        format!("{day} {month} {year}").trim().to_string()
    }

    /// dd/mm/yyyy format
    pub fn as_short_date_format(&self) -> String {
        // Day
        let day = match self.day() {
            Some(day) => format!("{day}"),
            None => String::from("-"),
        };

        // Month
        let month = match self.month() {
            Some(month) => format!("{month}"),
            None => String::from("-"),
        };

        // Year
        let year = format!("{}", self.year());

        // Return
        format!("{day} / {month} / {year}")
    }

    // TODO: pass in Option<Day>?
    /// Update an existing [`Date`]'s `day` if the result will be valid
    pub fn set_day(&mut self, day: Option<i64>) -> Result<(), DateError> {
        match day {
            None => self.day = None,
            Some(day) => {
                // Ensure the new Date will be valid
                let mut new_date = *self;
                new_date.day = Some(Day::try_from(day)?);
                new_date.is_valid()?;
                *self = new_date;
            }
        }
        Ok(())
    }

    // TODO: pass in Option<Month>?
    // TODO: this is wrong - can end up with a day but no month
    /// Update an existing [`Date`]'s `month` if the result will be valid
    pub fn set_month(&mut self, month: Option<i64>) -> Result<(), DateError> {
        match month {
            None => self.month = None,
            Some(month) => {
                // Ensure the new Date will be valid
                let mut new_date = *self;
                new_date.month = Some(Month::try_from(month)?);
                new_date.is_valid()?;
                *self = new_date;
            }
        }
        Ok(())
    }

    // TODO: pass in Option<Year>?
    /// Update an existing [`Date`]'s `year` if the result will be valid
    pub fn set_year(&mut self, year: i64) -> Result<(), DateError> {
        // Ensure the new Date will be valid
        let mut new_date = *self;
        new_date.year = Year::try_from(year)?;
        new_date.is_valid()?;
        *self = new_date;

        Ok(())
    }

    /// Get the [`Date`]'s day
    pub fn day(&self) -> Option<Day> {
        self.day
    }

    /// Get the [`Date`]'s month
    pub fn month(&self) -> Option<Month> {
        self.month
    }

    /// Get the [`Date`]'s year
    pub fn year(&self) -> Year {
        self.year
    }

    /// Check if the [`Date`] is valid
    fn is_valid(&self) -> Result<(), DateError> {
        match (self.day, self.month, self.year) {
            // Valid
            (None, None, _) => Ok(()),
            (None, Some(_), _) => Ok(()),
            (Some(_), Some(_), _) => Ok(()),

            // Not valid
            _ => Err(DateError::InvalidFields),
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.year.cmp(&other.year) {
            Ordering::Less => return Some(Ordering::Less),
            Ordering::Greater => return Some(Ordering::Greater),
            Ordering::Equal => (),
        };
        if let (Some(this_month), Some(other_month)) = (self.month, other.month) {
            match this_month.cmp(&other_month) {
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Equal => (),
            };
        } else {
            return None;
        }
        if let (Some(this_day), Some(other_day)) = (self.day, other.day) {
            match this_day.cmp(&other_day) {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => Some(Ordering::Equal),
            }
        } else {
            None
        }
    }
}

// Beware!
impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        let this_month = self.month().map(|m| m.value()).unwrap_or(1);
        let other_month = other.month().map(|m| m.value()).unwrap_or(1);

        let this_day = self.day().map(|d| d.value()).unwrap_or(1);
        let other_day = other.day().map(|d| d.value()).unwrap_or(1);

        (self.year, this_month, this_day).cmp(&(other.year, other_month, other_day))
    }
}

#[derive(Deserialize)]
struct RawDate {
    day: Option<i64>,
    month: Option<i64>,
    year: i64,
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: look into serde Visitors & doing without RawDate type
        let raw_date = RawDate::deserialize(deserializer)?;
        let date = Date::from(raw_date.day, raw_date.month, raw_date.year);
        match date {
            Ok(date) => Ok(date),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Date;

    #[test]
    fn from() {
        // Should return error
        assert!(Date::from(Some(1), None, 234).is_err());
        assert!(Date::from(None, None, 999_999).is_err());
        assert!(Date::from(None, None, -999_999).is_err());
        assert!(Date::from(Some(0), Some(0), 1234).is_err());
        assert!(Date::from(Some(32), Some(13), 1234).is_err());

        // Should be ok
        assert!(Date::from(Some(1), Some(1), 1).is_ok());
    }

    #[test]
    fn cmp() {
        // Year only
        let date_1 = Date::from(None, None, 234).unwrap();
        let date_2 = Date::from(None, None, 4321).unwrap();
        assert!(date_2 > date_1);
        assert!(date_1 < date_2);
        assert!(date_1 == date_1);
        assert!(date_1 != date_2);

        // Difference of 1 day
        let date_1 = Date::from(Some(1), Some(1), 234).unwrap();
        let date_2 = Date::from(Some(2), Some(1), 234).unwrap();
        assert!(date_2 > date_1);
    }
}
