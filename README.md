# roan
an example of using Rust on Android *and also* your Desktop.

Feel free to open a PR if you notice any words spelled poorly, my spellcheck
refuses to function.

## Setting up

### Grabbing the SDK/NDK

First off, you need the Android SDK and NDK. The SDK seems to be for linking
against anything Android, and the NDK for native, non-java code. On Arch Linux,
you can grab everything you need from the AUR by installing these packages
(default when asked):
`android-platform android-ndk android-sdk-build-tools`

- `android-platform` installs the SDK itself and a bunch of helpful tools.
- `android-ndk` is the Native Development Kit used for natvie code which, since
  we're writing Rust, is kind of neccessary :D

### Get the correct targets

What architecture is Android? When I think of "Android" I think of "the OS that
runs on my phone". That almost certainly means it's some kind of arm system.
There's a file in this repository, `install_targets.sh`, that will grab the
correct targets for Android on a real device as well as an emulated, x86 device.

Run `./install_targets.sh`

### Installing cargo-apk

`cargo-apk` is a wonderful little program for building apks and pushing them to
emulated systems (i haven't tried that yet!) or pushing them to a real device
via adb. Install it with `cargo install cargo-apk`.

cargo-apk is a wider part of [`android-ndk-rs`][android-ndk-rs], which is the
home key crates for the Rust on Android experience.

*Also also! Have a look at [cargo-mobile][cargo-mobile]! it's a similar thing*
*to cargo-apk, but it aims to support iOS too! I've yet to try it, but it*
*seems very interesting! Let me know your experience with it if you give*
*it a go.*

[cargo-mobile]: https://github.com/BrainiumLLC/cargo-mobile
[android-ndk-rs]: https://github.com/rust-windowing/android-ndk-rs

### Preparing your project

*this next section is almost entirely taken from the*
*[`android-ndk-rs`][android-ndk-rs] readme. You might want to go directly*
*read that [here][andk-hello]*

[andk-hello]: https://github.com/rust-windowing/android-ndk-rs#hello-world

From my understanding, the code you write in Rust sits behind a
`NativeActivity`, so your code is called as a library *from* that activity. The
`ndk-glue` crate manages this for us, but we have to do some work to get it there.

First, put this in your `Cargo.toml`:
```toml
[lib]
crate-type = ["lib", "cdylib"]
```

Now make a function in `src/lib.rs` and call it main for now. Be sure it's public
and add this attribute:
```rust
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
```

Go to your `src/main.rs` and call the lib's main from there, replacing `$crate`
with the name of your crate:
```rust
fn main() {
    $crate::main();
}
```

### It's all prepared now?

It's all prepared now! Mostly. Be sure to locate your SDK and NDK, mine are at
`/opt/android-sdk` and `/opt/android-ndk`, respectively.
Set the `ANDROID_SDK_ROOT` and `ANDROID_NDK_ROOT` environmental variables to the
paths of your sdk/ndk so that cargo-apk can find them.

Plug your phone in to your computer, using a cable that has a data connection (i've
used power-only cables too many times), and be sure it's there by running
`adb devices`. You should see one device listed. Be sure to accept the debug via USB
prompt, if it asks (it probably will).

You can now execute `cargo apk run` and the program will be compiled, uploaded to
your phone, and automatically ran.

But how can you know it ran? Check the logs! Run the command below to retreive them.
```bash
adb logcat RustStdoutStderr:D "*:S"
```

Within this repo, at the path `minimum`, is an example of everything I just described.
You can test that you're ready to start Rusting on Android if you go into that directory
and do run the code. Check the logs and you should get "Hello, Android!" as previously
mentioned.

Good luck :D