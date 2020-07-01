extern crate alloc_no_stdlib;
extern crate brotli;
extern crate core;
extern crate clap;
extern crate crypto;

extern crate num_cpus;
extern crate rand;
extern crate rustc_serialize;

extern crate rust_crypto;

use alloc_no_stdlib::{SliceWrapper, SliceWrapperMut, Allocator};

use brotli::enc::{UnionHasher, BrotliAlloc, BrotliEncoderParams, BrotliEncoderMaxCompressedSizeMulti};
use brotli::enc::threading::{
	SendAlloc,
	InternalSendAlloc,
	BatchSpawnable,
	BatchSpawnableLite,
	Joinable,
	Owned,
	OwnedRetriever,
	InternalOwned,
	BrotliEncoderThreadError,
	AnyBoxConstructor,
	PoisonedThreadError,
	CompressMulti
};

use core::marker::PhantomData;
use core::mem;
use core::ops;

use clap::{Arg, ArgMatches, App, SubCommand};

use crypto::aes;
use crypto::aes::KeySize;
use crypto::curve25519::{curve25519_base};

use num::Integer;

use rand::{thread_rng, Rng};
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};

use rust_crypto::{encrypt_buf, decrypt_buf};

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::result::Result;
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::vec::Vec;

///////////////////////////////////////////////////////////////////////////////

//--------------------------------------------------------
struct Rebox<T>
{
	b: Box<[T]>,
}

impl<T> From<Vec<T>> for Rebox<T>
{
	fn from(data: Vec<T>) -> Self
	{
		Rebox::<T>
		{
			b:data.into_boxed_slice()
		}
	}
}

impl<T> core::default::Default for Rebox<T>
{
	fn default() -> Self
	{
		let v: Vec<T> = Vec::new();
		let b = v.into_boxed_slice();
		Rebox::<T> { b: b }
	}
}

impl<T> ops::Index<usize> for Rebox<T>
{
	type Output = T;
	fn index(&self, index: usize) -> &T
	{
		&(*self.b)[index]
	}
}

impl<T> ops::IndexMut<usize> for Rebox<T>
{
	fn index_mut(&mut self, index: usize) -> &mut T
	{
		&mut (*self.b)[index]
	}
}

impl<T> alloc_no_stdlib::SliceWrapper<T> for Rebox<T>
{
	fn slice(&self) -> &[T]
	{
		&*self.b
	}
}

impl<T> alloc_no_stdlib::SliceWrapperMut<T> for Rebox<T>
{
	fn slice_mut(&mut self) -> &mut [T]
	{
		&mut *self.b
	}
}
//--------------------------------------------------------

//--------------------------------------------------------
struct MTJoinable<T: Send + 'static, U: Send + 'static>(JoinHandle<T>, PhantomData<U>);

#[cfg(not(feature="std"))]
impl<T: Send + 'static, U: Send + 'static + AnyBoxConstructor> Joinable<T, U> for MTJoinable<T, U>
{
	fn join(self) -> Result<T, U>
	{
		match self.0.join()
		{
			Ok(t) => Ok(t),
			Err(e) => Err(<U as AnyBoxConstructor>::new(e))
		}
	}
}

#[cfg(feature="std")]
impl<T: Send + 'static, U: Send + 'static + AnyBoxConstructor> Joinable<T, U> for MTJoinable<T, U>
{
	fn join(self) -> Result<T, U>
	{
		match self.0.join()
		{
			Ok(t) => Ok(t),
			Err(e) => Err(<U as AnyBoxConstructor>::new(e))
		}
	}
}
//--------------------------------------------------------

//--------------------------------------------------------
struct MTOwnedRetriever<U: Send + 'static>(std::sync::Arc<RwLock<U>>);

impl<U: Send + 'static> Clone for MTOwnedRetriever<U>
{
	fn clone(&self) -> Self
	{
		MTOwnedRetriever(self.0.clone())
	}
}

impl<U: Send + 'static> OwnedRetriever<U> for MTOwnedRetriever<U>
{
	fn view<T, F: FnOnce(&U) -> T>(&self, f: F) -> Result<T, PoisonedThreadError>
	{
		match self.0.read()
		{
			Ok(u) => Ok(f(&*u)),
			Err(_) => Err(PoisonedThreadError::default())
		}
	}

	fn unwrap(self) -> Result<U, PoisonedThreadError>
	{
		match std::sync::Arc::try_unwrap(self.0)
		{
			Ok(rwlock) => match rwlock.into_inner()
			{
				Ok(u) => Ok(u),
				Err(_) => Err(PoisonedThreadError::default())
			},
			Err(_) => Err(PoisonedThreadError::default())
		}
	}
}
//--------------------------------------------------------

//--------------------------------------------------------
#[derive(Default)]
struct MTSpawner{}

fn spawn_work<T: Send + 'static,
              ExtraInput:Send+'static,
              F: Fn(ExtraInput, usize, usize, &U, Alloc) -> T + Send + 'static,
              Alloc: BrotliAlloc + Send + 'static,
              U: Send + 'static + Sync>
	(extra_input: ExtraInput,
	 index: usize,
	 num_threads: usize,
	 locked_input: MTOwnedRetriever<U>, alloc: Alloc, f: F) -> std::thread::JoinHandle<T>
		where <Alloc as Allocator<u8>>::AllocatedMemory: Send + 'static
{
	std::thread::spawn(move ||
	{
		locked_input.view(move |guard: &U| -> T
		{
			f(extra_input, index, num_threads, guard, alloc)
		}).unwrap()
	})
}

impl<T: Send + 'static,
     ExtraInput: Send + 'static,
     Alloc: BrotliAlloc + Send + 'static,
     U: Send + 'static + Sync>
		BatchSpawnable<T, ExtraInput, Alloc, U>
		for MTSpawner
		where <Alloc as Allocator<u8>>::AllocatedMemory: Send + 'static
{
	type JoinHandle = MTJoinable<T, BrotliEncoderThreadError>;
	type FinalJoinHandle = MTOwnedRetriever<U>;

	fn make_spawner(&mut self, input: &mut Owned<U>) -> Self::FinalJoinHandle
	{
		MTOwnedRetriever(std::sync::Arc::<RwLock<U>>::new(RwLock::new(mem::replace(input, Owned(InternalOwned::Borrowed)).unwrap())))
	}

	fn spawn<F: Fn(ExtraInput, usize, usize, &U, Alloc) -> T + Send + 'static + Copy>(
		&mut self,
		locked_input: &mut Self::FinalJoinHandle,
		work: &mut SendAlloc<T, ExtraInput, Alloc, Self::JoinHandle>,
		index: usize,
		num_threads: usize,
		f: F)
	{
		let (alloc, extra_input) = work.replace_with_default();
		let ret = spawn_work(extra_input, index, num_threads, locked_input.clone(), alloc, f);
		*work = SendAlloc(InternalSendAlloc::Join(MTJoinable(ret, PhantomData::default())));
	}
}

impl<T: Send + 'static,
     ExtraInput: Send + 'static,
     Alloc: BrotliAlloc + Send + 'static,
     U: Send + 'static + Sync>
		BatchSpawnableLite<T, ExtraInput, Alloc, U>
		for MTSpawner
		where <Alloc as Allocator<u8>>::AllocatedMemory: Send + 'static
{
	type JoinHandle = <MTSpawner as BatchSpawnable<T, ExtraInput, Alloc, U>>::JoinHandle;
	type FinalJoinHandle = <MTSpawner as BatchSpawnable<T, ExtraInput, Alloc, U>>::FinalJoinHandle;

	fn make_spawner(&mut self, input: &mut Owned<U>) -> Self::FinalJoinHandle
	{
		<Self as BatchSpawnable<T, ExtraInput, Alloc, U>>::make_spawner(self, input)
	}

	fn spawn(&mut self,
	         handle: &mut Self::FinalJoinHandle,
	         alloc_per_thread: &mut SendAlloc<T, ExtraInput, Alloc, Self::JoinHandle>,
	         index: usize,
	         num_threads: usize,
	         f: fn(ExtraInput, usize, usize, &U, Alloc) -> T)
	{
		<Self as BatchSpawnable<T, ExtraInput, Alloc, U>>::spawn(self, handle, alloc_per_thread, index, num_threads, f)
	}
}
//--------------------------------------------------------

//--------------------------------------------------------
#[derive(Clone, Copy, Default)]
struct HeapAllocator
{
}

impl<T: core::clone:: Clone + Default> alloc_no_stdlib::Allocator<T> for HeapAllocator
{
	type AllocatedMemory = Rebox<T>;

	fn alloc_cell(self: &mut HeapAllocator, len: usize) -> Rebox<T>
	{
		let v: Vec<T> = vec![T::default();len];
		let b = v.into_boxed_slice();
		Rebox::<T> { b: b }
	}

	fn free_cell(self: &mut HeapAllocator, _data: Rebox<T>) {}
}

impl brotli::enc::BrotliAlloc for HeapAllocator
{
}
//--------------------------------------------------------

///////////////////////////////////////////////////////////////////////////////

fn main()
{
	let matches = App::new("cscrypto")
		.version("2.1.1")
		.author("© 2017-2019 by Tung Thanh Nguyen. <phonethanhtung@gmail.com>")
		.about("Public encrypt file")
		.subcommand(SubCommand::with_name("genkey")
			.about("Generate public key and private key")
			.arg(Arg::with_name("pub")
				.short("b")
				.long("pub")
				.required(true)
				.value_name("FILE")
				.help("File contains the public key"))
			.arg(Arg::with_name("pri")
				.short("p")
				.long("pri")
				.required(true)
				.value_name("FILE")
				.help("File contains the private key")))
		.subcommand(SubCommand::with_name("encrypt")
			.about("Encrypt the file with public key")
			.arg(Arg::with_name("level")
				.short("l")
				.long("level")
				.value_name("VALUE")
				.possible_values(&["medium", "high"])
				.default_value("medium")
				.help("Encrypt level"))
			.arg(Arg::with_name("pub")
				.short("p")
				.long("pub")
				.required(true)
				.value_name("FILE")
				.help("Public key file needed to encrypt file"))
			.arg(Arg::with_name("in")
				.short("i")
				.long("in")
				.required(true)
				.value_name("FILE")
				.help("File wants to encrypt"))
			.arg(Arg::with_name("out")
				.short("o")
				.long("out")
				.required(true)
				.value_name("FILE")
				.help("File was encrypted")))
		.subcommand(SubCommand::with_name("decrypt")
			.about("Decrypt the file with private key")
			.arg(Arg::with_name("pri")
				.short("p")
				.long("pri")
				.required(true)
				.value_name("FILE")
				.help("Private key file needed to decrypt file"))
			.arg(Arg::with_name("in")
				.short("i")
				.long("in")
				.required(true)
				.value_name("FILE")
				.help("Encrypted file"))
			.arg(Arg::with_name("out")
				.short("o")
				.long("out")
				.required(true)
				.value_name("FILE")
				.help("File was decrypted")))
		.get_matches();

	/////////////////////////////////////////////////////////////////////////////
	// Begin Generate key block

	if let Some(matches) = matches.subcommand_matches("genkey")
	{
		gen_key(matches);
	}

	// End Generate key block
	/////////////////////////////////////////////////////////////////////////////

	/////////////////////////////////////////////////////////////////////////////
	// Begin Encrypt block

	if let Some(matches) = matches.subcommand_matches("encrypt")
	{
		encrypt(matches);
	}

	// End Encrypt block
	/////////////////////////////////////////////////////////////////////////////

	/////////////////////////////////////////////////////////////////////////////
	// Begin Decrypt block

	if let Some(matches) = matches.subcommand_matches("decrypt")
	{
		decrypt(matches);
	}

	// End Decrypt block
	/////////////////////////////////////////////////////////////////////////////
}

///////////////////////////////////////////////////////////////////////////////



///////////////////////////////////////////////////////////////////////////////

fn gen_key(matches: &ArgMatches)
{
	let pub_file_name = match matches.value_of("pub")
	{
		Some(f) =>
		{
			if !f.is_empty()
			{
				let last_slash_index = match f.rfind('/')
				{
					Some(index) => index,
					None => 0
				};
				let dir_of: String = f.chars().take(last_slash_index).collect();
				if !dir_of.is_empty()
				{
					if !Path::new(&dir_of).exists()
					{
						println!("Directory for the public key file does not exist");
						return
					}
				}
			}
			f
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -b/--pub");
			return
		}
	};

	let pri_file_name = match matches.value_of("pri")
	{
		Some(f) =>
		{
			if !f.is_empty()
			{
				let last_slash_index = match f.rfind('/')
				{
					Some(index) => index,
					None => 0
				};
				let dir_of: String = f.chars().take(last_slash_index).collect();
				if !dir_of.is_empty()
				{
					if !Path::new(&dir_of).exists()
					{
						println!("Directory for the private key file does not exist");
						return
					}
				}
			}
			f
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -p/--pri");
			return
		}
	};

	let mut pub_file = match File::create(pub_file_name)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt create public key file named \"{}\"\n  For reason: {}", pub_file_name, why);
			return
		}
	};

	let mut pri_file = match File::create(pri_file_name)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt create private key file named \"{}\"\n  For reason: {}", pri_file_name, why);
			return
		}
	};

	// Begin Process generate public and private key pem...

	let mut private_key = [0u8; 32];
	thread_rng().fill(&mut private_key[..]);
	let public_key = curve25519_base(&private_key[..]);

	let _ = pub_file.write_all(public_key.to_base64(STANDARD).as_bytes());
	let _ = pri_file.write_all(private_key.to_base64(STANDARD).as_bytes());
	let _ = pub_file.sync_all();

	println!("Complete create public key in file \"{}\", and private key in file \"{}\"", pub_file_name, pri_file_name)

	// End Process generate public and private key pem...
}

fn encrypt(matches: &ArgMatches)
{
	let pub_file_path = match matches.value_of("pub")
	{
		Some(f) =>
		{
			if Path::new(f).exists() { f }
			else
			{
				println!("Public key file named \"{}\" does not exist", f);
				return
			}
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -p/--pub");
			return
		}
	};

	let in_file_path = match matches.value_of("in")
	{
		Some(f) =>
		{
			if Path::new(f).exists() { f }
			else
			{
				println!("Input file named \"{}\" does not exist", f);
				return
			}
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -i/--in");
			return
		}
	};

	let out_file_path = match matches.value_of("out")
	{
		Some(f) =>
		{
			if !f.is_empty()
			{
				let last_slash_index = match f.rfind('/')
				{
					Some(index) => index,
					None => 0
				};
				let dir_of: String = f.chars().take(last_slash_index).collect();
				if !dir_of.is_empty()
				{
					if !Path::new(&dir_of).exists()
					{
						println!("Directory for the output file does not exist");
						return
					}
				}
			}
			f
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -o/--out");
			return
		}
	};

	let enc_level: u8 = match matches.value_of("level").unwrap()
	{
		"medium" => 1,
		"high"   => 2,
		_        => 0
	};

	do_encrypt(pub_file_path, in_file_path, out_file_path, enc_level)
}

fn do_encrypt(pub_file_path: &str, in_file_path: &str, out_file_path: &str, enc_level: u8)
{
	let num_threads = num_cpus::get();

	let mut pub_file = match File::open(pub_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt open file with path \"{}\"\n  For reason: {}", pub_file_path, why);
			return
		}
	};

	let mut in_file = match File::open(in_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt open file with path \"{}\"\n  For reason: {}", in_file_path, why);
			return
		}
	};

	let mut out_file = match File::create(out_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt create output file with path \"{}\"\n  For reason: {}", out_file_path, why);
			return
		}
	};

	let mut public_pem = String::new();
	match pub_file.read_to_string(&mut public_pem)
	{
		Ok(fine) => fine,
		Err(why) =>
		{
			println!("Couldn't read public key from file with path \"{}\"\n  For reason: {}", pub_file_path, why);
			return
		}
	};
	let mut arr = [0u8; 32]; // temp buffer
	let public_key: &[u8; 32] = match public_pem.trim().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				println!("Public key size is wrong");
				return
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			println!("Public key is wrong\n  For reason: {}", why);
			return
		}
	};

	let brotli_compression_level = 11; // 0..11

	let mut buf: Vec<u8> = Vec::new();
	in_file.read_to_end(&mut buf).unwrap();

	// println!("Number of threads = {}", num_threads);

	match enc_level
	{
		1 => // Medium level
		{
			// Begin Process encrypt medium operation...

			// Generate random key

			let mut gen = thread_rng();
			let mut key = vec![0u8; 128];
			gen.fill(key.as_mut_slice());
			let mut iv = vec![0u8; 128];
			gen.fill(iv.as_mut_slice());

			// End Generate random key

			// println!("Length of raw data = {}\n", buf.len());

			let mut buf_compressed = Rebox::from(vec![0u8; BrotliEncoderMaxCompressedSizeMulti(buf.len(), num_threads)]);
			let _ = brotli_compress_multi_nostd(buf, buf_compressed.slice_mut(), brotli_compression_level, num_threads, false);

			// println!("\nlength of buf_compressed = {}\n", buf_compressed.len());
			// println!("\nAES key = {:?}\n\n    iv = {:?}\n", key, iv);

			let mut cipher = aes::ctr(KeySize::KeySize1024, key.as_slice(), iv.as_slice());
			let mut enc_dat = vec![0u8; buf_compressed.len()];
			cipher.process(buf_compressed.slice(), enc_dat.as_mut_slice());

			key.append(&mut iv);
			let mut enc_key: Vec<u8> = match encrypt_buf(&public_key, &key)
			{
				Ok(k) => k,
				Err(_) =>
				{
					println!("Couldn't encrypt a key");
					return
				}
			};
			let mut magic_num: u16 = enc_dat[0] as u16;
			magic_num = magic_num.gcd(&(enc_dat[1] as u16)) + (enc_key.len() as u16);
			enc_dat.append(&mut enc_key);
			enc_dat.extend_from_slice(&magic_num.to_le_bytes());

			let mut magic_num2: u16 = enc_dat[0] as u16;
			magic_num2 = magic_num2.gcd(&(enc_dat[1] as u16)) + 1; // 1 is indicator of medium encrypt level
			enc_dat.extend_from_slice(&magic_num2.to_le_bytes());

			let mut buf_compressed2 = Rebox::from(vec![0u8; BrotliEncoderMaxCompressedSizeMulti(enc_dat.len(), num_threads)]);
			let _ = brotli_compress_multi_nostd(enc_dat, buf_compressed2.slice_mut(), brotli_compression_level, num_threads, false);
			let _ = out_file.write_all(buf_compressed2.slice().to_base64(STANDARD).as_bytes());

			println!("Completed encrypt file \"{}\" to file \"{}\" in medium level", in_file_path, out_file_path);

			// End Process encrypt medium operation...
		},
		2 => // High level
		{
			// Begin Process encrypt high operation...

			let mut buf_compressed = Rebox::from(vec![0u8; BrotliEncoderMaxCompressedSizeMulti(buf.len(), num_threads)]);
			let _ = brotli_compress_multi_nostd(buf, buf_compressed.slice_mut(), brotli_compression_level, num_threads, false);

			let mut enc_dat: Vec<u8> = match encrypt_buf(&public_key, &buf_compressed.slice())
			{
				Ok(d) => d,
				Err(_) =>
				{
					println!("Couldn't encrypt a buffer");
					return
				}
			};
			let magic_num: u8 = enc_dat[0].gcd(&enc_dat[1]) + 2; // 2 is indicator of high encrypt level
			enc_dat.push(magic_num);

			let mut buf_compressed2 = Rebox::from(vec![0u8; BrotliEncoderMaxCompressedSizeMulti(enc_dat.len(), num_threads)]);
			let _ = brotli_compress_multi_nostd(enc_dat, buf_compressed2.slice_mut(), brotli_compression_level, num_threads, false);

			let _ = out_file.write_all(buf_compressed2.slice().to_base64(STANDARD).as_bytes());
			let _ = out_file.sync_all();

			println!("Completed encrypt file \"{}\" to file \"{}\" in high level", in_file_path, out_file_path)

			// End Process encrypt high operation...
		},
		_ => unreachable!()
	}
}

fn decrypt(matches: &ArgMatches)
{
	let pri_file_path = match matches.value_of("pri")
	{
		Some(f) =>
		{
			if Path::new(f).exists() { f }
			else
			{
				println!("Private key file named \"{}\" does not exist", f);
				return
			}
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -p/--pri");
			return
		}
	};

	let in_file_path = match matches.value_of("in")
	{
		Some(f) =>
		{
			let path = Path::new(f);
			if path.exists() { f }
			else
			{
				println!("Input file named \"{}\" does not exist", f);
				return
			}
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -i/--in");
			return
		}
	};

	let out_file_path = match matches.value_of("out")
	{
		Some(f) =>
		{
			if !f.is_empty()
			{
				let last_slash_index = match f.rfind('/')
				{
					Some(index) => index,
					None => 0
				};
				let dir_of: String = f.chars().take(last_slash_index).collect();
				if !dir_of.is_empty()
				{
					if !Path::new(&dir_of).exists()
					{
						println!("Directory for the output file does not exist");
						return
					}
				}
			}
			f
		},
		None =>
		{
			println!("Error occurated when getting value for parameter -o/--out");
			return
		}
	};

	do_decrypt(pri_file_path, in_file_path, out_file_path)
}

fn do_decrypt(pri_file_path: &str, in_file_path: &str, out_file_path: &str)
{
	const BROTLI_BUFFER_SIZE: usize = 4096;

	// Begin Process decrypt operation...

	let mut pri_file = match File::open(pri_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt open input file with path \"{}\"\n  For reason: {}", pri_file_path, why);
			return
		}
	};

	let mut in_file = match File::open(in_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt open input file with path \"{}\"\n  For reason: {}", in_file_path, why);
			return
		}
	};

	let mut out_file = match File::create(out_file_path)
	{
		Ok(done) => done,
		Err(why) =>
		{
			println!("Couldnʼt create file with path \"{}\"\n  For reason: {}", out_file_path, why);
			return
		}
	};

	let mut private_pem = String::new();
	match pri_file.read_to_string(&mut private_pem)
	{
		Ok(fine) => fine,
		Err(why) =>
		{
			println!("Couldn't read private key from file with path \"{}\"\n  For reason: {}", pri_file_path, why);
			return
		}
	};
	let mut arr = [0u8; 32]; // temp buff
	let private_key: &[u8; 32] = match private_pem.trim().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				println!("Private key size is wrong");
				return
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			println!("Private key is wrong\n  For reason: {}", why);
			return
		}
	};

	let mut enc_msg = String::new();
	match in_file.read_to_string(&mut enc_msg)
	{
		Ok(fine) => fine,
		Err(why) =>
		{
			println!("Couldn't read encrypted data from file named \"{}\"\n  For reason: {}", in_file_path, why);
			return
		}
	};
	let buf: Vec<u8> = enc_msg.trim().from_base64().unwrap();
	// println!("buf size = {}", buf.len());

	let mut buf_decompressed: Vec<u8> = Vec::new();

	let mut brotli_reader = brotli::Decompressor::new(buf.as_slice(), BROTLI_BUFFER_SIZE);
	let mut buf_tmp = [0u8; BROTLI_BUFFER_SIZE];
	loop
	{
		match brotli_reader.read(&mut buf_tmp[..])
		{
			Err(why) =>
			{
				println!("Couldn't decompress raw data\n  For reason: {}", why);
				return
			}
			Ok(size) =>
			{
				if size == 0 { break }
				buf_decompressed.extend_from_slice(&buf_tmp[..size])
			}
		}
	}

	// println!("buf_decompressed.len() = {}", buf_decompressed.len());
	// Get encrypt level
	if buf_decompressed.len() < 4
	// if buf.len() < 4
	{
		println!("Size of encrypted file is wrong");
		return
	}
	let encrypt_level_raw: [u8; 2] = [buf_decompressed.pop().unwrap(), buf_decompressed.pop().unwrap()];
	let mut encrypt_level: u16 = u16::from_be_bytes(encrypt_level_raw);
	encrypt_level = encrypt_level - (buf_decompressed[0].gcd(&buf_decompressed[1]) as u16);
	// let encrypt_level: u8 = buf.pop().unwrap() - buf[0].gcd(&buf[1]);
	// println!("encrypt_level = {}", encrypt_level);
	// End Get encrypt level

	if encrypt_level == 1 // Encrypt level is medium
	{
		// println!("Encrypt level is medium");

		let enc_key_len_raw: [u8; 2] = [buf_decompressed.pop().unwrap(), buf_decompressed.pop().unwrap()];
		let mut enc_key_len: usize = u16::from_be_bytes(enc_key_len_raw) as usize;
		enc_key_len = enc_key_len - (buf_decompressed[0].gcd(&buf_decompressed[1]) as usize);
		let mut buf_len = buf_decompressed.len();
		let enc_key = buf_decompressed.split_off(buf_len - enc_key_len);
		// let enc_key_len: usize = (buf.pop().unwrap() - buf[0].gcd(&buf[1])) as usize;
		// let mut buf_len = buf.len();
		// let enc_key = buf.split_off(buf_len - enc_key_len);
		let mut key: Vec<u8> = match decrypt_buf(&private_key, &enc_key[..])
		{
			Ok(k) => k,
			Err(_) =>
			{
				println!("Couldn't decrypt a key");
				return
			}
		};
		buf_len = key.len();
		let iv = key.split_off(buf_len - 128);

		let mut cipher = aes::ctr(KeySize::KeySize1024, key.as_slice(), iv.as_slice());
		let mut dec_dat = vec![0u8; buf_decompressed.len()];
		cipher.process(buf_decompressed.as_slice(), dec_dat.as_mut_slice());
		// let mut dec_dat = vec![0u8; buf.len()];
		// cipher.process(buf.as_slice(), dec_dat.as_mut_slice());

		// println!("AES key = {:?}\n\n    iv = {:?}\n", key, iv);
		// println!("\nlength of dec_dat = {}\n", dec_dat.len());

		let mut buf_decompressed2: Vec<u8> = Vec::new();

		let mut brotli_reader = brotli::Decompressor::new(dec_dat.as_slice(), BROTLI_BUFFER_SIZE);
		let mut buf_tmp = [0u8; BROTLI_BUFFER_SIZE];
		let mut counter = 0;
		loop
		{
			counter = counter + 1;
			match brotli_reader.read(&mut buf_tmp[..])
			{
				Err(why) =>
				{
					println!("Couldn't decompress raw data\n  For reason: {}", why);
					return
				}
				Ok(size) =>
				{
					if size == 0 { break }
					buf_decompressed2.extend_from_slice(&buf_tmp[..size])
				}
			}
		}

		// println!("\ncounter = {}\n", counter);
		// println!("\nLength of raw data = {}\n", buf_decompressed2.len());

		let _ = out_file.write_all(&buf_decompressed2);
		// let byte_writed = out_file.write(&dec_dat);
		let _ = out_file.sync_all();

		// println!("\nNumber of byte is writed to file = {:?}\n", byte_writed);
	}
	else if encrypt_level == 2 // Encrypt level is high
	{
		// println!("Encrypt level is high");

		let dec_dat: Vec<u8> = match decrypt_buf(&private_key, &buf_decompressed[..])
		// let dec_dat: Vec<u8> = match decrypt(&private_key, &buf[..])
		{
			Ok(d) => d,
			Err(_) =>
			{
				println!("Couldn't decrypt data");
				return
			}
		};

		let mut buf_decompressed2: Vec<u8> = Vec::new();

		let mut brotli_reader = brotli::Decompressor::new(dec_dat.as_slice(), BROTLI_BUFFER_SIZE);
		let mut buf_tmp = [0u8; BROTLI_BUFFER_SIZE];
		loop
		{
			match brotli_reader.read(&mut buf_tmp[..])
			{
				Err(why) =>
				{
					println!("Couldn't decompress raw data\n  For reason: {}", why);
					return
				}
				Ok(size) =>
				{
					if size == 0 { break }
					buf_decompressed2.extend_from_slice(&buf_tmp[..size])
				}
			}
		}

		let _ = out_file.write_all(&buf_decompressed2);
		// let byte_writed = out_file.write(&dec_dat);
		let _ = out_file.sync_data();

		// println!("\nNumber of byte is writed to file = {:?}\n", byte_writed);
	}

	println!("Completed decrypt file \"{}\" to file \"{}\"", in_file_path, out_file_path)

	// End Process decrypt operation...
}

fn brotli_compress_multi_nostd(input: Vec<u8>, output: &mut [u8], quality: i32, num_threads: usize, catable: bool) -> Result<usize, BrotliEncoderThreadError>
{
	let mut params = BrotliEncoderParams::default();
	params.quality = quality;
	params.magic_number = true;
	if catable
	{
		params.catable = true;
		params.use_dictionary = false;
	}

	let mut alloc_per_thread = Vec::new();
	for _ in 1..=num_threads
	{
		alloc_per_thread.push(SendAlloc::new(HeapAllocator::default(), UnionHasher::Uninit))
	}

	CompressMulti(&params, &mut Owned::new(Rebox::from(input)), output, &mut alloc_per_thread[..num_threads], &mut MTSpawner::default())
}

///////////////////////////////////////////////////////////////////////////////