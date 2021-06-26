// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cc;

use std::env;
use std::path::Path;

fn main()
{
	let target = env::var("TARGET").unwrap();
	let host = env::var("HOST").unwrap();
	if target.contains("msvc") && host.contains("windows")
	{
		let mut config = cc::Build::new();
		config.file("src/util_helpers.asm");
		config.file("src/aesni_helpers.asm");
		if target.contains("x86_64")
		{
			config.define("X64", None);
		}
		config.compile("lib_rust_crypto_helpers.a")
	}
	else
	{
		let mut cfg = cc::Build::new();
		cfg.file("src/util_helpers.c");
		cfg.file("src/aesni_helpers.c");
		if env::var_os("CC").is_none()
		{
			if host.contains("openbsd")
			{
				// Use clang on openbsd since there have been reports that
				// GCC doesn't like some of the assembly that we use on that
				// platform.
				cfg.compiler(Path::new("clang"));
			}
			else
			{
				if target.contains("android")
				{
					if target.contains("aarch64") { cfg.compiler(Path::new("aarch64-linux-android-clang")); }
					else if target.contains("arm") { cfg.compiler(Path::new("arm-linux-androideabi-clang")); }
					else if target.contains("i686") { cfg.compiler(Path::new("i686-linux-android-clang")); }
					else if target.contains("x86_64") { cfg.compiler(Path::new("x86_64-linux-android-clang")); }
					env::set_var("AR", "llvm-ar");
				}
				else if target.contains("wasm32")
				{
					env::set_var("CC", "/mnt/PrivateData/formatlibs/wasi-sdk-12.0/bin/clang");
					env::set_var("AR", "/mnt/PrivateData/formatlibs/wasi-sdk-12.0/bin/llvm-ar");
					cfg.flag("--target=wasm32-wasi");
					cfg.flag("--sysroot=/mnt/PrivateData/formatlibs/wasi-sdk-12.0/share/wasi-sysroot");
				}
				else
				{
					cfg.compiler(Path::new("cc"));
				}
			}
		}
		cfg.compile("lib_rust_crypto_helpers.a");
	}
}
