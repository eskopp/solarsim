//! The Sun and eight planets at epoch J2000.0.
//!
//! `mass` is in solar masses (planet figures fold in their moons where it
//! matters, so "Earth" is really the Earth-Moon barycentre). `radius_km` is
//! only used to size the spheres. `elements` are mean Keplerian elements on
//! the J2000 ecliptic, from JPL's "Approximate Positions of the Major
//! Planets": semi-major axis (AU), eccentricity, inclination, mean longitude,
//! longitude of perihelion and longitude of the ascending node (all degrees).

pub struct Elements {
    pub a: f64,
    pub e: f64,
    pub i: f64,
    pub l: f64,
    pub varpi: f64,
    pub omega: f64,
}

pub struct Body {
    pub name: &'static str,
    /// RGB, 0..1 per channel.
    pub color: [f32; 3],
    pub mass: f64,
    pub radius_km: f64,
    /// `None` for the Sun, which starts at rest in the origin.
    pub elements: Option<Elements>,
}

const fn el(a: f64, e: f64, i: f64, l: f64, varpi: f64, omega: f64) -> Option<Elements> {
    Some(Elements {
        a,
        e,
        i,
        l,
        varpi,
        omega,
    })
}

pub fn bodies() -> Vec<Body> {
    vec![
        Body {
            name: "Sun",
            color: [1.00, 0.81, 0.30],
            mass: 1.0,
            radius_km: 696_000.0,
            elements: None,
        },
        Body {
            name: "Mercury",
            color: [0.69, 0.64, 0.60],
            mass: 1.66012e-7,
            radius_km: 2_440.0,
            elements: el(
                0.38709927,
                0.20563593,
                7.00497902,
                252.25032350,
                77.45779628,
                48.33076593,
            ),
        },
        Body {
            name: "Venus",
            color: [0.85, 0.72, 0.55],
            mass: 2.44780e-6,
            radius_km: 6_052.0,
            elements: el(
                0.72333566,
                0.00677672,
                3.39467605,
                181.97909950,
                131.60246718,
                76.67984255,
            ),
        },
        Body {
            name: "Earth",
            color: [0.42, 0.58, 0.84],
            mass: 3.04043e-6,
            radius_km: 6_371.0,
            elements: el(
                1.00000261,
                0.01671123,
                -0.00001531,
                100.46457166,
                102.93768193,
                0.0,
            ),
        },
        Body {
            name: "Mars",
            color: [0.76, 0.27, 0.05],
            mass: 3.22715e-7,
            radius_km: 3_390.0,
            elements: el(
                1.52371034,
                0.09339410,
                1.84969142,
                -4.55343205,
                -23.94362959,
                49.55953891,
            ),
        },
        Body {
            name: "Jupiter",
            color: [0.85, 0.79, 0.62],
            mass: 9.54792e-4,
            radius_km: 69_911.0,
            elements: el(
                5.20288700,
                0.04838624,
                1.30439695,
                34.39644051,
                14.72847983,
                100.47390909,
            ),
        },
        Body {
            name: "Saturn",
            color: [0.92, 0.84, 0.72],
            mass: 2.85886e-4,
            radius_km: 58_232.0,
            elements: el(
                9.53667594,
                0.05386179,
                2.48599187,
                49.95424423,
                92.59887831,
                113.66242448,
            ),
        },
        Body {
            name: "Uranus",
            color: [0.62, 0.84, 0.87],
            mass: 4.36624e-5,
            radius_km: 25_362.0,
            elements: el(
                19.18916464,
                0.04725744,
                0.77263783,
                313.23810451,
                170.95427630,
                74.01692503,
            ),
        },
        Body {
            name: "Neptune",
            color: [0.29, 0.44, 0.87],
            mass: 5.15139e-5,
            radius_km: 24_622.0,
            elements: el(
                30.06992276,
                0.00859048,
                1.77004347,
                -55.12002969,
                44.96476227,
                131.78422574,
            ),
        },
    ]
}
