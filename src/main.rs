//! solarsim — an N-body solar system integrated live and drawn with macroquad.
//!
//! Camera: drag (or one finger) to orbit, wheel (or pinch) to zoom.
//! Everything else is on-screen buttons, so it works on a phone too.

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

fn next_method(m: Method) -> Method {
    match m {
        Method::Euler => Method::SymplecticEuler,
        Method::SymplecticEuler => Method::Verlet,
        Method::Verlet => Method::Euler,
    }
}

/// The on-screen control column, so everything works without a keyboard.
const UI_W: f32 = 138.0;
const UI_TOP: f32 = 14.0;
const UI_ROW: f32 = 46.0;
const UI_ROWS: usize = 6;

fn ui_x() -> f32 {
    screen_width() - UI_W - 12.0
}

fn pointer_over_ui(p: Vec2) -> bool {
    p.x >= ui_x() - 4.0 && p.y <= UI_TOP + UI_ROW * UI_ROWS as f32
}

/// Draw a button, return true on the frame it is tapped or clicked.
fn button(x: f32, y: f32, w: f32, label: &str, active: bool) -> bool {
    let h = 40.0;
    let p = Vec2::from(mouse_position());
    let hovered = p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h;
    let bg = if active {
        Color::new(0.18, 0.36, 0.84, 0.92)
    } else {
        Color::new(0.07, 0.10, 0.17, 0.88)
    };
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.35, 0.45, 0.65, 1.0));
    let d = measure_text(label, None, 18, 1.0);
    draw_text(
        label,
        x + (w - d.width) / 2.0,
        y + h / 2.0 + d.offset_y / 2.0,
        18.0,
        WHITE,
    );
    hovered && is_mouse_button_pressed(MouseButton::Left)
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
    let mut last_mouse = Vec2::from(mouse_position());
    let mut last_pinch: Option<f32> = None;

    loop {
        // --- camera: wheel + drag, or one / two finger touch -----------
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            dist = (dist * 1.15_f32.powf(-wheel_y.signum())).clamp(3.0, 500.0);
        }

        let fingers = touches();
        if fingers.len() >= 2 {
            let spread = fingers[0].position.distance(fingers[1].position);
            if let Some(prev) = last_pinch {
                if spread > 1.0 {
                    dist = (dist * prev / spread).clamp(3.0, 500.0);
                }
            }
            last_pinch = Some(spread);
        } else {
            last_pinch = None;
            let m = Vec2::from(mouse_position());
            if is_mouse_button_down(MouseButton::Left) && !pointer_over_ui(m) {
                yaw -= (m.x - last_mouse.x) * 0.005;
                pitch = (pitch + (m.y - last_mouse.y) * 0.005).clamp(-1.45, 1.45);
            }
        }
        last_mouse = Vec2::from(mouse_position());

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
        set_camera(&Camera3D {
            position: vec3(
                dist * cp * yaw.cos(),
                dist * pitch.sin(),
                dist * cp * yaw.sin(),
            ),
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

        // --- HUD text -------------------------------------------------
        set_default_camera();
        let (yr, mo, dy) = time::date_since_j2000(sim.time);
        let lines = [
            format!("{yr:05}-{mo:02}-{dy:02}   ({:+.0} d from J2000)", sim.time),
            format!("integrator      {}", sim.method.name()),
            format!("step  {dt} d      speed  {steps_per_frame}x/frame"),
            format!("energy drift     {:+.2e}", sim.energy_drift()),
            format!("ang. mom drift   {:+.2e}", sim.ang_mom_drift()),
            (if playing { "running" } else { "PAUSED" }).to_string(),
        ];
        for (i, line) in lines.iter().enumerate() {
            draw_text(
                line,
                14.0,
                24.0 + i as f32 * 20.0,
                19.0,
                Color::new(0.85, 0.9, 1.0, 1.0),
            );
        }

        // --- on-screen controls (mouse and touch) -------------------
        let bx = ui_x();
        let mut by = UI_TOP;
        if button(
            bx,
            by,
            UI_W,
            if playing { "Pause" } else { "Play" },
            playing,
        ) {
            playing = !playing;
        }
        by += UI_ROW;
        if button(bx, by, UI_W, sim.method.short(), true) {
            sim.method = next_method(sim.method);
        }
        by += UI_ROW;
        if button(bx, by, UI_W, "Trails", show_trails) {
            show_trails = !show_trails;
        }
        by += UI_ROW;
        let half = UI_W / 2.0 - 3.0;
        if button(bx, by, half, "Slower", false) {
            steps_per_frame = steps_per_frame.saturating_sub(1).max(1);
        }
        if button(bx + UI_W / 2.0 + 3.0, by, half, "Faster", false) {
            steps_per_frame = (steps_per_frame + 1).min(200);
        }
        by += UI_ROW;
        if button(bx, by, half, "Step /2", false) {
            dt = (dt * 0.5).max(1.0 / 32.0);
        }
        if button(bx + UI_W / 2.0 + 3.0, by, half, "Step x2", false) {
            dt = (dt * 2.0).min(32.0);
        }
        by += UI_ROW;
        if button(bx, by, UI_W, "Reset", false) {
            let method = sim.method;
            sim = Sim::new(&bodies);
            sim.method = method;
            for t in &mut trails {
                t.clear();
            }
        }

        // --- planet legend -----------------------------------------
        let step = ((screen_width() - 28.0) / bodies.len() as f32).min(96.0);
        for (i, b) in bodies.iter().enumerate() {
            draw_text(
                b.name,
                14.0 + i as f32 * step,
                screen_height() - 14.0,
                18.0,
                colors[i],
            );
        }

        next_frame().await
    }
}
