// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use std::ops::RangeInclusive;

// TODO: this needs testing
/// Alter the string that's passed referenced so that it holds a string
/// representation of a number in the specified range.
///
/// There is no "-0".
pub fn conform_string_input_to_int_in_range(str: &mut String, range: RangeInclusive<isize>) {
    // Filter the string to remove all chars that are not ASCII numeric (with
    // the exception of a '-' char at index 0).
    let filtered_str = str
        .chars()
        .enumerate()
        .filter(|(i, c)| c.is_ascii_digit() || ((*c == '-') && (*i == 0)))
        .map(|(_, c)| c)
        .collect();
    *str = filtered_str;

    // If negative numbers are allowed, replace "-0" with "-"
    if range.clone().min().is_none() || range.clone().min().is_some_and(|min| min < 0) {
        if str == "-" {
            return;
        } else if str == "-0" {
            *str = "-".to_string();
            return;
        }
    }

    // Parse the string to an integer.  If the integer isn't in the accepted
    // range, remove the last char
    if let Ok(value) = str.parse::<isize>() {
        *str = value.to_string();
        if !(range).contains(&value) {
            str.pop();
        }
    }
}
