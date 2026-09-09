# solarsim

The Sun and eight planets, integrated as a full N-body gravitational system in
real time and drawn with [macroquad](https://macroquad.rs). Written in Rust,
compiled to WebAssembly, served from GitHub Pages — the same binary also runs
natively on Linux, Windows and macOS.

**Live:** https://eskopp.github.io/solarsim/

## Controls

| | |
|---|---|
| drag | orbit camera |
| wheel | zoom |
| space | pause / resume |
| `1` `2` `3` | Euler / Symplectic Euler / Velocity Verlet |
| `[` `]` | halve / double the time step |
| `-` `=` | fewer / more steps per frame |
| `T` | toggle orbit trails |
| `R` | reset to J2000 |

## How it works

| File | Job |
|------|-----|
| `src/bodies.rs` | Masses and J2000 Keplerian elements from JPL |
| `src/kepler.rs` | Elements → heliocentric state vector, once, at t = 0 |
| `src/physics.rs` | Pairwise Newtonian gravity + three integrators; energy and angular momentum |
| `src/time.rs` | Julian Date → calendar date for the clock |
| `src/main.rs` | macroquad render loop, camera, HUD |

The learning payload is in `physics.rs`: switch the integrator at runtime and
watch the **energy drift** readout. Plain Euler runs away within a few orbits;
symplectic Euler stays bounded but wobbles; velocity Verlet holds energy to
~1e-6 over a decade. The unit tests assert exactly that.

## Build

```sh
cargo run --release                                    # native window
cargo test                                             # physics checks

rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/solarsim.wasm web/
cd web && python -m http.server 8000                   # http://localhost:8000
```

CI builds the WASM on every push to `main` and publishes `web/` + the `.wasm`
to Pages.

## Limitations

- Newtonian point masses only — no general relativity (so Mercury's perihelion
  precession is missing its famous 43″/century), no non-spherical gravity, no
  non-gravitational forces.
- Planet spheres are wildly exaggerated in size; distances are to scale.
- Initial state is mean elements, not a JPL ephemeris fit, so it drifts from
  reality over long spans.

## License

MIT (`LICENSE`). macroquad is MIT/Apache-2.0.
