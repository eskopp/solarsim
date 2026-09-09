//! solarsim — an N-body solar system integrated live and drawn with macroquad.
//!
//! Controls:
//!   drag         orbit camera            wheel   zoom
//!   space        pause / resume          R       reset to J2000
//!   1 / 2 / 3    Euler / Symplectic / Verlet
//!   [ / ]        time step  -/+          - / =   speed -/+
//!   T            toggle orbit trails

mod bodies;
mod kepler;
mod physics;
mod time;
mod vec;

use macroquad::prelude::*;

use bodies::Body;
use physics::{Method, Sim};

const MAX_TRAIL: usize = 3000;

fn window_conf() -> Conf {
    Conf {
        window_title: "solarsim".to_owned(),
        window_width: 1200,
        window_height: 800,
        high_dpi: true,
        ..Default::default()
    }
}

fn v3(v: vec::V3) -> Vec3 {
    vec3(v.x as f32, v.y as f32, v.z as f32)
}

fn display_radius(radius_km: f64, is_sun: bool) -> f32 {
    if is_sun {
        return 1.4;
    }
    (0.15 * (radius_km / 6371.0).powf(0.45)).clamp(0.05, 0.9) as f32
}

#[macroquad::main(window_conf)]
async fn main() {
    let bodies: Vec<Body> = bodies::bodies();
    let radii: Vec<f32> = bodies
        .iter()
        .enumerate()
        .map(|(i, b)| display_radius(b.radius_km, i == 0))
        .collect();
    let colors: Vec<Color> = bodies
        .iter()
        .map(|b| Color::new(b.color[0], b.color[1], b.color[2], 1.0))
        .collect();

    let mut sim = Sim::new(&bodies);
    let mut trails: Vec<Vec<Vec3>> = vec![Vec::new(); bodies.len()];

    let mut playing = true;
    let mut show_trails = true;
    let mut dt = 1.0_f64; // days per integration step
    let mut steps_per_frame = 4usize;

    let mut yaw = 0.7_f32;
    let mut pitch = 0.5_f32;
    let mut dist = 85.0_f32;
    let mut last_mouse = mouse_position();

    loop {
        // --- input ---------------------------------------------------------
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        if is_key_pressed(KeyCode::T) {
            show_trails = !show_trails;
        }
        if is_key_pressed(KeyCode::Key1) {
            sim.method = Method::Euler;
        }
        if is_key_pressed(KeyCode::Key2) {
            sim.method = Method::SymplecticEuler;
        }
        if is_key_pressed(KeyCode::Key3) {
            sim.method = Method::Verlet;
        }
        if is_key_pressed(KeyCode::LeftBracket) {
            dt = (dt * 0.5).max(0.03125);
        }
        if is_key_pressed(KeyCode::RightBracket) {
            dt = (dt * 2.0).min(32.0);
        }
        if is_key_pressed(KeyCode::Minus) {
            steps_per_frame = (steps_per_frame.saturating_sub(1)).max(1);
        }
        if is_key_pressed(KeyCode::Equal) {
            steps_per_frame = (steps_per_frame + 1).min(200);
        }
        if is_key_pressed(KeyCode::R) {
            let method = sim.method;
            sim = Sim::new(&bodies);
            sim.method = method;
            for t in &mut trails {
                t.clear();
            }
        }

        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            dist *= 1.15_f32.powf(-wheel_y.signum());
            dist = dist.clamp(3.0, 500.0);
        }
        let mouse = mouse_position();
        if is_mouse_button_down(MouseButton::Left) {
            yaw -= (mouse.0 - last_mouse.0) * 0.005;
            pitch = (pitch + (mouse.1 - last_mouse.1) * 0.005).clamp(-1.45, 1.45);
        }
        last_mouse = mouse;

        // --- step --------------------------------------------------------
        if playing {
            for _ in 0..steps_per_frame {
                sim.step(dt);
            }
            for (i, trail) in trails.iter_mut().enumerate() {
                let p = v3(sim.pos[i]);
                if trail.last().is_none_or(|&q| q.distance(p) > 1e-4) {
                    trail.push(p);
                    if trail.len() > MAX_TRAIL {
                        trail.remove(0);
                    }
                }
            }
        }

        // --- draw 3D ----------------------------------------------------
        clear_background(Color::new(0.02, 0.03, 0.05, 1.0));

        let cp = pitch.cos();
        let eye = vec3(
            dist * cp * yaw.cos(),
            dist * pitch.sin(),
            dist * cp * yaw.sin(),
        );
        set_camera(&Camera3D {
            position: eye,
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            fovy: 45.0_f32.to_radians(),
            ..Default::default()
        });

        if show_trails {
            for (i, trail) in trails.iter().enumerate() {
                let mut c = colors[i];
                c.a = 0.5;
                for w in trail.windows(2) {
                    draw_line_3d(w[0], w[1], c);
                }
            }
        }
        for i in 0..bodies.len() {
            draw_sphere(v3(sim.pos[i]), radii[i], None, colors[i]);
        }

        // --- draw HUD -------------------------------------------------
        set_default_camera();
        let (y, m, d) = time::date_since_j2000(sim.time);
        let lines = [
            format!("{y:05}-{m:02}-{d:02}    ({:+.0} d from J2000)", sim.time),
            format!("integrator   {}   [1/2/3]", sim.method.name()),
            format!("step         {dt} d   [ / ]        speed  {steps_per_frame}x   - / ="),
            format!("energy drift    {:+.2e}", sim.energy_drift()),
            format!("ang. mom drift  {:+.2e}", sim.ang_mom_drift()),
            format!(
                "{}   trails {}   space pause   R reset",
                if playing { "running" } else { "PAUSED" },
                if show_trails { "on" } else { "off" }
            ),
        ];
        for (i, line) in lines.iter().enumerate() {
            draw_text(
                line,
                16.0,
                26.0 + i as f32 * 22.0,
                22.0,
                Color::new(0.85, 0.9, 1.0, 1.0),
            );
        }
        for (i, b) in bodies.iter().enumerate() {
            let x = 16.0 + i as f32 * 92.0;
            draw_text(b.name, x, screen_height() - 16.0, 20.0, colors[i]);
        }

        next_frame().await
    }
}
