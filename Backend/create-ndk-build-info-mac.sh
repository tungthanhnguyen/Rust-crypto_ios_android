#!/bin/sh

echo 'Creating the build tools and information...'

BUILD_TOOLS_PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
ln -f "$BUILD_TOOLS_PATH/aarch64-linux-android21-clang" "$BUILD_TOOLS_PATH/aarch64-linux-android-clang"
ln -f $BUILD_TOOLS_PATH/i686-linux-android16-clang $BUILD_TOOLS_PATH/i686-linux-android-clang
ln -f $BUILD_TOOLS_PATH/x86_64-linux-android21-clang $BUILD_TOOLS_PATH/x86_64-linux-android-clang

sed 's|$PWD|'"${ANDROID_NDK_HOME}"'|g' cargo-config.toml.template.new.mac > cargo-config.toml
