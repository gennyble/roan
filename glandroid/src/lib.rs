#![cfg(target_os = "android")]
mod app;

#[ndk_glue::main()]
pub fn main() {
    app::run();
}
