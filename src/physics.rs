//! The N-body core: Newtonian gravity between every pair of bodies, advanced
//! with one of three integrators so the difference is visible.
//!
//! Units: AU, days, solar masses. In these units G is the square of the
//! Gaussian gravitational constant.

use crate::bodies::Body;
use crate::kepler::state_from_elements;
use crate::vec::V3;

pub const K: f64 = 0.01720209895;
pub const G: f64 = K * K;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Explicit Euler. Energy runs away fast — that is the point.
    Euler,
    /// Semi-implicit (symplectic) Euler. Energy stays bounded but wobbles.
    SymplecticEuler,
    /// Velocity Verlet. Symplectic, 2nd order, the sane default.
    Verlet,
}

impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Method::Euler => "Euler",
            Method::SymplecticEuler => "Symplectic Euler",
            Method::Verlet => "Velocity Verlet",
        }
    }
}

pub struct Sim {
    pub mass: Vec<f64>,
    pub pos: Vec<V3>,
    pub vel: Vec<V3>,
    acc: Vec<V3>,
    pub time: f64, // days since J2000
    pub method: Method,
    pub energy0: f64,
    pub ang_mom0: f64,
}

impl Sim {
    pub fn new(bodies: &[Body]) -> Self {
        let sun_mass = bodies[0].mass;
        let mut mass = Vec::with_capacity(bodies.len());
        let mut pos = Vec::with_capacity(bodies.len());
        let mut vel = Vec::with_capacity(bodies.len());

        for b in bodies {
            mass.push(b.mass);
            match &b.elements {
                None => {
                    pos.push(V3::ZERO);
                    vel.push(V3::ZERO);
                }
                Some(el) => {
                    let (p, v) = state_from_elements(el, G * (sun_mass + b.mass));
                    pos.push(p);
                    vel.push(v);
                }
            }
        }

        let mut sim = Sim {
            acc: vec![V3::ZERO; bodies.len()],
            mass,
            pos,
            vel,
            time: 0.0,
            method: Method::Verlet,
            energy0: 0.0,
            ang_mom0: 0.0,
        };
        sim.zero_barycentre_drift();
        sim.recompute_acc();
        sim.energy0 = sim.energy();
        sim.ang_mom0 = sim.angular_momentum().len();
        sim
    }

    fn n(&self) -> usize {
        self.mass.len()
    }

    /// Shift into the barycentric frame so the whole system does not sail off.
    fn zero_barycentre_drift(&mut self) {
        let mut p = V3::ZERO;
        let mut m = 0.0;
        for i in 0..self.n() {
            m += self.mass[i];
            p += self.vel[i].scale(self.mass[i]);
        }
        let v = p.scale(1.0 / m);
        for vi in &mut self.vel {
            *vi = *vi - v;
        }
    }

    fn accel_at(&self, pos: &[V3], out: &mut [V3]) {
        for o in out.iter_mut() {
            *o = V3::ZERO;
        }
        for i in 0..self.n() {
            for j in (i + 1)..self.n() {
                let d = pos[j] - pos[i];
                let r2 = d.dot(d);
                let inv = 1.0 / r2.sqrt();
                let inv3 = inv * inv * inv;
                out[i] += d.scale(G * self.mass[j] * inv3);
                out[j] += d.scale(-G * self.mass[i] * inv3);
            }
        }
    }

    fn recompute_acc(&mut self) {
        let mut a = std::mem::take(&mut self.acc);
        self.accel_at(&self.pos, &mut a);
        self.acc = a;
    }

    pub fn step(&mut self, dt: f64) {
        match self.method {
            Method::Euler => self.step_euler(dt),
            Method::SymplecticEuler => self.step_symplectic(dt),
            Method::Verlet => self.step_verlet(dt),
        }
        self.time += dt;
    }

    fn step_euler(&mut self, dt: f64) {
        self.recompute_acc();
        for i in 0..self.n() {
            let v = self.vel[i];
            self.pos[i] += v.scale(dt);
            self.vel[i] += self.acc[i].scale(dt);
        }
    }

    fn step_symplectic(&mut self, dt: f64) {
        self.recompute_acc();
        for i in 0..self.n() {
            self.vel[i] += self.acc[i].scale(dt);
            let v = self.vel[i];
            self.pos[i] += v.scale(dt);
        }
    }

    fn step_verlet(&mut self, dt: f64) {
        for i in 0..self.n() {
            let half = self.acc[i].scale(0.5 * dt);
            self.vel[i] += half;
            let v = self.vel[i];
            self.pos[i] += v.scale(dt);
        }
        let mut next = vec![V3::ZERO; self.n()];
        self.accel_at(&self.pos, &mut next);
        for (v, a) in self.vel.iter_mut().zip(next.iter()) {
            *v += a.scale(0.5 * dt);
        }
        self.acc = next;
    }

    pub fn energy(&self) -> f64 {
        let mut ke = 0.0;
        for i in 0..self.n() {
            ke += 0.5 * self.mass[i] * self.vel[i].dot(self.vel[i]);
        }
        let mut pe = 0.0;
        for i in 0..self.n() {
            for j in (i + 1)..self.n() {
                pe -= G * self.mass[i] * self.mass[j] / (self.pos[j] - self.pos[i]).len();
            }
        }
        ke + pe
    }

    pub fn angular_momentum(&self) -> V3 {
        let mut l = V3::ZERO;
        for i in 0..self.n() {
            l += self.pos[i].cross(self.vel[i]).scale(self.mass[i]);
        }
        l
    }

    /// Signed relative energy error since t = 0.
    pub fn energy_drift(&self) -> f64 {
        (self.energy() - self.energy0) / self.energy0.abs()
    }

    pub fn ang_mom_drift(&self) -> f64 {
        (self.angular_momentum().len() - self.ang_mom0) / self.ang_mom0.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::bodies;

    fn run(method: Method, years: f64, dt: f64) -> Sim {
        let mut sim = Sim::new(&bodies());
        sim.method = method;
        let steps = (years * 365.25 / dt) as usize;
        for _ in 0..steps {
            sim.step(dt);
        }
        sim
    }

    #[test]
    fn solar_system_is_bound() {
        let sim = Sim::new(&bodies());
        assert!(sim.energy0 < 0.0, "total energy should be negative");
    }

    #[test]
    fn verlet_conserves_energy() {
        // 0.25 d steps resolve even Mercury's perihelion well.
        let sim = run(Method::Verlet, 10.0, 0.25);
        assert!(
            sim.energy_drift().abs() < 1e-6,
            "verlet energy drift too large: {}",
            sim.energy_drift()
        );
    }

    #[test]
    fn angular_momentum_is_conserved() {
        let sim = run(Method::Verlet, 10.0, 1.0);
        assert!(sim.ang_mom_drift().abs() < 1e-10);
    }

    #[test]
    fn plain_euler_drifts_much_worse_than_verlet() {
        let euler = run(Method::Euler, 10.0, 1.0).energy_drift().abs();
        let verlet = run(Method::Verlet, 10.0, 1.0).energy_drift().abs();
        assert!(euler > 100.0 * verlet, "euler {euler}, verlet {verlet}");
    }

    #[test]
    fn earth_returns_after_one_year() {
        let mut sim = Sim::new(&bodies());
        let start = sim.pos[3];
        for _ in 0..365 {
            sim.step(1.0);
        }
        // A calendar year is ~365.25 days, so a day short the Earth is close
        // but not exactly back — within a few million km.
        let gap = (sim.pos[3] - start).len();
        assert!(gap < 0.05, "earth is {gap} AU from where it started");
    }
}
