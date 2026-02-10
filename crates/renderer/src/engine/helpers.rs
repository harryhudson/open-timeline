// SPDX-License-Identifier: MIT

//!
//! Helper functions
//!

// TODO: do we need this?  The JavaScript version had some entity flickering as
// they switched rows as we dragged.  This I think fixed it.  However, we no
// longer calculate positions/rows when the timeline is dragged.
/// Round an f64 value to the nearest 0.1.  Once we have some huge timelines
/// with lots of entities, we must check this.  I thought it was down to the
/// imprecision of floats, but I'm not so conviced by this anymore.
pub(crate) fn round_f64_to_nearest_0_1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

///
pub(crate) fn floor_to_decade(value: i32) -> i32 {
    if value < 0 {
        ((value - 9) / 10) * 10
    } else {
        (value / 10) * 10
    }
}

///
pub(crate) fn ceiling_to_decade(value: i32) -> i32 {
    if value < 0 {
        (value / 10) * 10
    } else {
        ((value + 9) / 10) * 10
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_floor_to_decade() {
        assert_eq!(floor_to_decade(-150), -150);
        assert_eq!(floor_to_decade(-151), -160);
        assert_eq!(floor_to_decade(-159), -160);
        assert_eq!(floor_to_decade(150), 150);
        assert_eq!(floor_to_decade(151), 150);
        assert_eq!(floor_to_decade(159), 150);
    }

    #[test]
    fn test_ceiling_to_decade() {
        assert_eq!(ceiling_to_decade(-150), -150);
        assert_eq!(ceiling_to_decade(-151), -150);
        assert_eq!(ceiling_to_decade(-159), -150);
        assert_eq!(ceiling_to_decade(150), 150);
        assert_eq!(ceiling_to_decade(151), 160);
        assert_eq!(ceiling_to_decade(159), 160);
    }
}
