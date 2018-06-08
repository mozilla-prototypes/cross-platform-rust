# Toodle: cross-platform, Rust-powered TODO app

This is an example project demonstrating Rust-based generic storage and syncing for iOS, Android and Firefox's WebExtension.

# iOS
## Building

First, we have to [install Xcode](https://itunes.apple.com/us/app/xcode/id497799835?ls=1&amp;mt=12) and then set up Xcode build tools. If you already have the build tools installed and they are up to date, you can skip this step.

```
xcode-select --install
```

Next, we need to ensure that Rust is installed and that we can cross compile to the iOS architectures. For this we will be using [rustup](https://www.rustup.rs/). If you already have rustup installed, you can skip this step. Rustup installs Rust from the official release channels and enables you to easily switch between different release versions. It will be useful to you for all your future Rust development, not just here.
```
curl https://sh.rustup.rs -sSf | sh
```

Add the iOS architectures to rustup so we can use them during cross compilation.
```
rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
```

When you installed Rust, it also installed cargo, which is a package manager similar to pip, gems etc. Now we will use cargo to install `cargo-lipo`. This is a cargo subcommand which automatically creates a universal library for use with iOS. Without this crate, cross compiling Rust to work on iOS is infinitely harder.
```
cargo install cargo-lipo
```

We need to build our library against the iOS architectures using `cargo-lipo`. The built artifacts of will be placed in `rust/target/`. The universal iOS library that we are interested in can be found in `rust/target/universal/release/libtoodle.a`.

```
cd rust
cargo lipo --release
```

We are now using [Carthage](https://github.com/Carthage/Carthage) to import external dependencies. Install Carthage using the [instructions](https://github.com/Carthage/Carthage#installing-carthage) on their site. Once installed, open a terminal and navigate to the iOS project root. To install the project dependencies, run the following command:

```
carthage update
```

Open `ios/Toodle/Toodle.xcodeproj` in Xcode. Select the Toodle project from the project navigator, and then ensure the Toodle target is selected. Open the `General` tab. Scroll down to the `Linked Frameworks and Libraries` section. Import your `libtoodle_ffi.a` library by either dragging it in from Finder, or clicking the `+` at the bottom of the list, clicking 'Add other…' and navigating to `rust/target/universal/release/`. Select `libtoodle_ffi.a` and then click `Open`.

You should now be able to build and run your iOS app.

# Android
{TODO}
* Configure Rust build environment as per https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html
* Build for all architectures via helper script `rust/android_build_all.sh`
* Load the `android/Toodle` project in Android Studio, and run it on an API26 emulator

# WebExtension
[WebExtensions](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/What_are_WebExtensions) can't directly bind to native code. Instead, we use the [native messaging API](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_messaging) to send requests to a helper application, or "bridge", that embeds the Toodle library. The bridge manages a single `Toodle` store, translates requests into method calls, and communicates with the WebExtension via length-prefixed JSON messages over standard input and output. The WebExtension runtime automatically launches and terminates the bridge.

To build the bridge:

```
cd toodlext
cargo update
cargo build
```

The WebExtension runtime requires a [native messaging manifest](https://developer.mozilla.org/en-US/Add-ons/WebExtensions/Native_manifests) to allow the WebExtension to talk to the bridge. There's a helper Node script that installs a native manifest for a debug build of the bridge on macOS. This only needs to be done once. Please note that Windows and Linux use different locations, but the install script doesn't handle them yet.

```
cd webextension
./install-native-manifest.js
```

Then, to launch the WebExtension in a dedicated Firefox profile:

```
npm install
npm run dev
```

This also watches for source file changes, rebuilds, and automatically reloads the extension.
