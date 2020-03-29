#!/bin/sh

set -eu

if [ -d NDK ]; then
    printf '\033[33;1mStandalone NDK already exists... Delete the NDK folder to make a new one.\033[0m\n\n'
    printf '  $ rm -rf NDK\n'
    exit 0
fi

if [ ! -d "${ANDROID_SDK_ROOT-}" ]; then
    ANDROID_SDK_ROOT=$ANDROID_SDK
fi
if [ ! -d "${ANDROID_HOME-}" ]; then
    ANDROID_HOME="$ANDROID_SDK_ROOT"
fi
if [ ! -d "${ANDROID_NDK_HOME-}" ]; then
    ANDROID_NDK_HOME="$ANDROID_HOME/ndk-bundle"
fi
MAKER="${ANDROID_NDK_HOME}/build/tools/make_standalone_toolchain.py"

if [ -x "$MAKER" ]; then
    echo 'Creating standalone NDK...'
else
    printf '\033[91;1mPlease install Android NDK!\033[0m\n\n'
    printf '  $ sdkmanager ndk-bundle\n\n'
    printf "\033[33;1mnote\033[0m: file \033[34;4m$MAKER\033[0m not found.\n"
    printf 'If you have installed the NDK in non-standard location, please define the \033[1m$ANDROID_NDK_HOME\033[0m variable.\n'
    exit 1
fi

NDK_STANDALONE_HOME="$HOME/NDK"
if [ -d "$NDK_STANDALONE_HOME" ]; then
	rm -rf "$NDK_STANDALONE_HOME"
fi
mkdir "$NDK_STANDALONE_HOME"

create_ndk() {
    echo "($1)..."
    "$MAKER" --api "$2" --arch "$1" --install-dir "$NDK_STANDALONE_HOME/$1"
}

create_ndk arm64 21
create_ndk arm 16
create_ndk x86 16
create_ndk x86_64 21

ln -s $ANDROID_NDK_HOME/toolchains/llvm $NDK_STANDALONE_HOME/llvm

FILES=("$NDK_STANDALONE_HOME/arm/bin/arm-linux-androideabi-g++" "$NDK_STANDALONE_HOME/arm/bin/arm-linux-androideabi-gcc" "$NDK_STANDALONE_HOME/arm64/bin/aarch64-linux-android-g++" "$NDK_STANDALONE_HOME/arm64/bin/aarch64-linux-android-gcc" "$NDK_STANDALONE_HOME/x86/bin/i686-linux-android-g++" "$NDK_STANDALONE_HOME/x86/bin/i686-linux-android-gcc")
for file in "${FILES[@]}"; do
	cat "$file" | sed -e 's/\/..\/..\/..\/..\//\/..\/..\//' > "$file".edt
	mv "$file".edt "$file"
	chmod 775 "$file"
done

echo 'Updating cargo-config.toml...'

sed 's|$PWD|'"${HOME}"'|g' cargo-config.toml.template > cargo-config.toml

