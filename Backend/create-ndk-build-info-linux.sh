#!/bin/sh

echo 'Creating the build tools and information...'

BUILD_TOOLS_PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
ln -f "$BUILD_TOOLS_PATH/aarch64-linux-android21-clang" "$BUILD_TOOLS_PATH/aarch64-linux-android-clang"
ln -f $BUILD_TOOLS_PATH/armv7a-linux-androideabi16-clang $BUILD_TOOLS_PATH/arm-linux-androideabi-clang
ln -f $BUILD_TOOLS_PATH/i686-linux-android16-clang $BUILD_TOOLS_PATH/i686-linux-android-clang
ln -f $BUILD_TOOLS_PATH/x86_64-linux-android21-clang $BUILD_TOOLS_PATH/x86_64-linux-android-clang

sed 's|$PWD|'"${ANDROID_NDK_HOME}"'|g' cargo-config.toml.template.new.linux > cargo-config.toml
