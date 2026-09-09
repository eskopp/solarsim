//! Julian Date -> calendar date, for the on-screen clock.

const J2000_JD: f64 = 2_451_545.0; // 2000-01-01 12:00 TT

/// (year, month, day) for a number of days since J2000.
pub fn date_since_j2000(days: f64) -> (i64, u32, u32) {
    let jd = J2000_JD + days;
    let z = (jd + 0.5).floor() as i64;
    let a = if z < 2_299_161 {
        z
    } else {
        let alpha = ((z as f64 - 1_867_216.25) / 36_524.25).floor() as i64;
        z + 1 + alpha - alpha / 4
    };
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d) - (30.6001 * e as f64).floor() as i64;
    let month = if e < 14 { e - 1 } else { e - 13 };
    let year = if month > 2 { c - 4716 } else { c - 4715 };
    (year, month as u32, day as u32)
}
