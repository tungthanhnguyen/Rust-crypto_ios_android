rem @echo off

rem i686 x86_64 aarch64
SET BUILD_ARCH=%1

rem debug release
SET BUILD_TYPE=%2

SET LIB_RUST_NAME=rust_crypto
SET RUST_TARGET=%BUILD_ARCH%-pc-windows-msvc
SET CORELIB_DIR=..\..\..\Backend\%LIB_RUST_NAME%
SET CARGO_COMMAND=%USERPROFILE%\.cargo\bin\cargo.exe

IF %BUILD_TYPE% == debug (
	%CARGO_COMMAND% build --target %RUST_TARGET% --manifest-path %CORELIB_DIR%\Cargo.toml
) ELSE (
	%CARGO_COMMAND% build --target %RUST_TARGET% --%BUILD_TYPE% --manifest-path %CORELIB_DIR%\Cargo.toml
)

copy %CORELIB_DIR%\target\%RUST_TARGET%\%BUILD_TYPE%\%LIB_RUST_NAME%.lib %CORELIB_DIR%\target