# glandrooid

This project demonstrates using OpenGL on Android via the `glow` crate
([github][glow-gh], [crates.io][glow-crates]), which is
a crate that provides abstract bindings over OpenGL, OpenGL ES, and WebGL. We'll
only take advantage of the first two with this example.

[glow-gh]: https://github.com/grovesNL/glow
[glow-crates]: https://crates.io/crates/glow

## A quick note on glutin/winit

I originally learned how to OpenGL with Rust on Android by inspecting a repository
called `game-gl`. It uses a fork of glutin and winit that have fixes to run on
Android. The owner of that repository left this note about glutin/winit in the readme
of game-gl. [Read it][gamegl-glutin] if you care, but otherwise note that you should
use the glutin fork that's referenced in the `Cargo.toml`.

[gamegl-glutin]: https://github.com/Kaiser1989/game-gl#why-dont-you-create-pull-requests-in-the-original-projects