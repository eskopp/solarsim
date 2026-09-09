//! Kepler orbital elements -> heliocentric state vector, once, at t = J2000.
//! After this the elements are never touched again: the N-body integrator
//! takes over from the initial positions and velocities.

use crate::bodies::Elements;
use crate::vec::V3;

const DEG: f64 = std::f64::consts::PI / 180.0;

/// Solve `M = E - e sin E` for the eccentric anomaly `E` (radians).
fn solve_kepler(mut m: f64, e: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    m = m.rem_euclid(tau);
    if m > std::f64::consts::PI {
        m -= tau;
    }
    let mut ecc = if e < 0.8 { m } else { std::f64::consts::PI };
    for _ in 0..60 {
        let d = (ecc - e * ecc.sin() - m) / (1.0 - e * ecc.cos());
        ecc -= d;
        if d.abs() < 1e-14 {
            break;
        }
    }
    ecc
}

/// `mu` is the gravitational parameter G(M_sun + m_planet) in AU^3/day^2.
/// Returns (position AU, velocity AU/day) on the J2000 ecliptic.
pub fn state_from_elements(el: &Elements, mu: f64) -> (V3, V3) {
    let a = el.a;
    let e = el.e;
    let inc = el.i * DEG;
    let node = el.omega * DEG;
    let arg = (el.varpi - el.omega) * DEG; // argument of perihelion
    let mean = (el.l - el.varpi) * DEG; // mean anomaly

    let ecc = solve_kepler(mean, e);
    let (se, ce) = ecc.sin_cos();
    let b = (1.0 - e * e).sqrt();

    // Perifocal frame: x toward perihelion, motion counter-clockwise.
    let px = a * (ce - e);
    let py = a * b * se;
    let f = (mu / a).sqrt() / (1.0 - e * ce);
    let vx = f * -se;
    let vy = f * b * ce;

    // Rotate perifocal -> ecliptic: Rz(node) * Rx(inc) * Rz(arg).
    let (sa, ca) = arg.sin_cos();
    let (sn, cn) = node.sin_cos();
    let (si, ci) = inc.sin_cos();

    let r = [
        [cn * ca - sn * sa * ci, -cn * sa - sn * ca * ci, sn * si],
        [sn * ca + cn * sa * ci, -sn * sa + cn * ca * ci, -cn * si],
        [sa * si, ca * si, ci],
    ];
    let rot = |x: f64, y: f64| {
        V3::new(
            r[0][0] * x + r[0][1] * y,
            r[1][0] * x + r[1][1] * y,
            r[2][0] * x + r[2][1] * y,
        )
    };
    (rot(px, py), rot(vx, vy))
}
