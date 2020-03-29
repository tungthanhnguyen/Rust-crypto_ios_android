Rust-crypto_ios_android (was supported for arm64)
================

<img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/iOS.png" height=600 /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/Android.png" height=600 /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/macOS.png" /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/Linux.png" /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/Win32.png" /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/Windows-UWP.png" /> <img src="https://raw.githubusercontent.com/tungthanhnguyen/Rust-crypto_ios_android/master/Screenshoots/WebAssembly.png" />

Example project for building a library for iOS + Android in Rust. macOS is
required for iOS development.

* ✓ Rust 1.42.0
* ✓ Android 7.1 – 10.0 (API 25 – 29)
* ✓ Swift 5.1
* ✓ iOS 11.4 – 13.3

*Note: The purpose of this project is not to create a pure Rust app, but rather
use Rust as a shared native component between the mobile platforms.*

You may also want to check <https://github.com/Geal/rust_on_mobile>.

Setup
-----

1. Install the common build tools like C compiler and linker. On macOS, get
    Xcode, and install the command line tools.

    ```sh
    xcode-select --install
    ```

2. Get Android NDK. We recommend installing it via Android Studio or
    `sdkmanager`:

    ```sh
    sdkmanager --verbose ndk-bundle
    ```

    Otherwise, please define the environment variable `ANDROID_NDK_HOME` to the
    path of the current version of Android NDK.

    ```sh
    export ANDROID_NDK_HOME='/usr/local/opt/android-ndk/android-ndk-r21/'
    ```

3. Create the build information for NDK.

    ```sh
    # Linux.
    ./create-ndk-build-info-linux.sh

    # Mac.
    ./create-ndk-build-info-mac.sh
    ```

4. Add the NDK to search path:

    ```sh
    # Linux.
    export PATH="$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

    # Mac.
    export PATH="$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
    ```

5. Download [rustup](https://www.rustup.rs/). We will use this to setup Rust for
   cross-compiling.

    ```sh
    curl https://sh.rustup.rs -sSf | sh
    ```

6. Download targets for iOS and Android.

    ```sh
    # iOS.
    rustup target add aarch64-apple-ios x86_64-apple-ios

    # Android.
    rustup target add aarch64-linux-android i686-linux-android x86_64-linux-android
    ```

7. Copy the content of `cargo-config.toml` (consists of linker information of
   the Android targets) to `~/.cargo/config`

    ```sh
    cat cargo-config.toml >> ~/.cargo/config
    ```

8. Install cargo-lipo to generate the iOS universal library.

    ```sh
    cargo install cargo-lipo
    ```

Creating the libraries
----------------------

You use use the `iOS/`, `Android/` projects as an example. (Note that the sample itself
does not contain proper error checking.)

1. Write the library and expose a C interface. See [the FFI chapter in the Rust
   Book](http://doc.rust-lang.org/book/ffi.html) for an introduction.

2. Expose the Java interface with JNI when `target_os="android"`.

3. Build the libraries.

    ```sh
    cd Backend/rust_crypto

    # iOS.
    # Integrated into Xcode project.

    # Android.
    cargo build --target aarch64-linux-android --release
    cargo build --target i686-linux-android --release
    cargo build --target x86_64-linux-android --release
    ```

4. Build the Xcode project.

    Using Xcode 11.4

    When you create an Xcode project yourself, note the following points:
    * Add a new `Run Script` phase to your `Build Phases`. Place it before `Compile Sources`. Add something like the following to the script:

    ```sh
    export PATH="$HOME/.cargo/bin:$PATH"

    cargo lipo --release --manifest-path ../../Backend/rust_crypto/Cargo.toml
    ```
    * Build the project once, then update the `Link Binary with Libraries` phase: Click the `+`, then choose `Add Other...`. Navigate to `Backend/rust_crypto/target/universal/release` and select file `librust_crypto.a`.
    * You need to link to `libresolv.tbd`.
    * Go back to your `Build Settings` and add `Library Search Paths` for `Debug` and `Release`, pointing to `../../Backend/rust_crypto/target/universal/release`.
    * Add the C header `rust_crypto.h` to allow using the Rust functions from C.
    * Note that `cargo-lipo` does not generate bitcode yet. You must set
      `ENABLE_BITCODE` to `NO`. (See also <http://stackoverflow.com/a/38488617>)

5. Build the Android project.

    Using Android Studio 3.5.3

    When you create an Android Studio project yourself, note the following
    points:

    * Copy or link the `*.so` into the corresponding `src/main/jniLibs` folders:

        Copy from Rust | Copy to Android
        ---|---
        `target/aarch64-linux-android/release/lib???.so` | `src/main/jniLibs/arm64-v8a/lib???.so`
        `target/i686-linux-android/release/lib???.so` | `src/main/jniLibs/x86/lib???.so`
        `target/x86_64-linux-android/release/lib???.so` | `src/main/jniLibs/x86_64/lib???.so`

    * Don't forget to ensure the JNI glue between Rust and Java are compatible.
