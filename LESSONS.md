# Lessons Learned

Read this file before implementation and again before final review. Record only verified,
recurring traps that are not already obvious from code, tests, or canonical documentation.
Update an existing lesson instead of adding a duplicate.

## Keep desktop engine features at the presentation boundary

- Context: The initial root package depended on Bevy 0.19 with all default features.
- Symptom: `cargo clippy` on a headless Linux/WSL2 environment failed while building
  `wayland-sys` because the `wayland-client` system package was unavailable.
- Cause: Bevy's default platform features pulled windowing, Wayland, X11, input, and audio
  dependencies into every repository check before any project code was analyzed.
- Resolution: Move Bevy into `dreadstep-bevy`, disable default features, and enable only
  `std` until a presentation milestone needs a reviewed feature set. A representative
  Bevy 0.19 package with this configuration passed Clippy on the same environment.
- Prevention: Keep engine dependencies out of core, protocol, and content; inspect enabled
  features before adding presentation capabilities; verify the headless Linux workflow.
