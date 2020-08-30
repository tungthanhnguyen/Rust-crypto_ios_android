extern crate crypto;
extern crate rand;
extern crate rustc_serialize;

use crypto::aead::{AeadEncryptor, AeadDecryptor};
use crypto::chacha20poly1305::ChaCha20Poly1305;
use crypto::curve25519::{curve25519_base, curve25519};

use rand::{thread_rng, Rng};
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};

use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use std::ptr::null_mut;
use std::vec::Vec;

///////////////////////////////////////////////////////////////////////////////
// Begin Extern functions

#[no_mangle]
pub extern fn rust_encrypt(pub_key: *const c_char, msg: *const c_char) -> *const c_char
{
	let c_pub_tmp: &CStr = unsafe { CStr::from_ptr(pub_key) };
	let mut arr = [0u8; 32]; // temp buffer
	let c_pub_key: &[u8; 32] = match c_pub_tmp.to_bytes().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				println!("Public key size is wrong");
				return null_mut()
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			println!("Public key is wrong\n  For reason: {}", why);
			return null_mut()
		}
	};
	let c_msg: &CStr = unsafe { CStr::from_ptr(msg) };

	let enc_dat: Vec<u8> = match encrypt_buf(&c_pub_key, c_msg.to_bytes())
	{
		Ok(d) => d,
		Err(_) =>
		{
			println!("Couldn't encrypt data");
			return null_mut()
		}
	};

	let result = CString::new(enc_dat.as_slice().to_base64(STANDARD).as_bytes()).unwrap();
	result.into_raw()
}

#[no_mangle]
pub extern fn rust_decrypt(pri_key: *const c_char, msg: *const c_char) -> *const c_char
{
	let c_pri_tmp: &CStr = unsafe { CStr::from_ptr(pri_key) };
	let mut arr = [0u8; 32]; // temp buffer
	let c_pri_key: &[u8; 32] = match c_pri_tmp.to_bytes().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				println!("Private key size is wrong");
				return null_mut()
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			println!("Private key is wrong\n  For reason: {}", why);
			return null_mut()
		}
	};
	let c_msg: &CStr = unsafe { CStr::from_ptr(msg) };
	let recv_msg: Vec<u8> = match c_msg.to_bytes().from_base64()
	{
		Ok(r) => r,
		Err(_) =>
		{
			println!("Invalid Base64 string");
			return null_mut()
		}
	};

	let dec_dat: Vec<u8> = match decrypt_buf(&c_pri_key, &recv_msg)
	{
		Ok(d) => d,
		Err(_) =>
		{
			println!("Couldn't decrypt data");
			return null_mut()
		}
	};

	let result = CString::new(dec_dat).unwrap();
	result.into_raw()
}

// End Extern functions
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
// Begin For JNI

/// Expose the JNI interface for android below
#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android
{
	extern crate jni;

	use super::*;
	use self::jni::JNIEnv;
	use self::jni::objects::{JClass, JString};
	use self::jni::sys::jstring;

	#[no_mangle]
	pub unsafe extern fn Java_com_comtasoft_cryptodemo_MainActivity_RustEncrypt(env: JNIEnv, _: JClass, java_pub_key: JString, java_msg: JString) -> jstring
	{
		let pub_key = env.get_string(java_pub_key).expect("invalid pub_key string");
		let msg = env.get_string(java_msg).expect("invalid msg string");

		let c_pub_tmp: &CStr = CStr::from_ptr(pub_key.as_ptr());
		let mut arr = [0u8; 32]; // temp buffer
		let c_pub_key: &[u8; 32] = match c_pub_tmp.to_bytes().from_base64()
		{
			Ok(m) =>
			{
				if m.len() != 32
				{
					println!("Public key size is wrong");
					return null_mut()
				}
				arr.copy_from_slice(&m);
				&arr
			},
			Err(why) =>
			{
				println!("Public key is wrong\n  For reason: {}", why);
				return null_mut()
			}
		};
		let c_msg: &CStr = CStr::from_ptr(msg.as_ptr());

		let enc_dat: Vec<u8> = match encrypt_buf(&c_pub_key, c_msg.to_bytes())
		{
			Ok(d) => d,
			Err(_) =>
			{
				println!("Couldn't encrypt data");
				return null_mut()
			}
		};

		let result = enc_dat.as_slice().to_base64(STANDARD);

		let output = env.new_string(result).expect("Couldn't create java string!");
		output.into_inner()
	}

	#[no_mangle]
	pub unsafe extern fn Java_com_comtasoft_cryptodemo_MainActivity_RustDecrypt(env: JNIEnv, _: JClass, java_pri_key: JString, java_msg: JString) -> jstring
	{
		let pri_key = env.get_string(java_pri_key).expect("invalid pri_key string");
		let msg = env.get_string(java_msg).expect("invalid msg string");

		let c_pri_tmp: &CStr = CStr::from_ptr(pri_key.as_ptr());
		let mut arr = [0u8; 32]; // temp buffer
		let c_pri_key: &[u8; 32] = match c_pri_tmp.to_bytes().from_base64()
		{
			Ok(m) =>
			{
				if m.len() != 32
				{
					println!("Private key size is wrong");
					return null_mut()
				}
				arr.copy_from_slice(&m);
				&arr
			},
			Err(why) =>
			{
				println!("Private key is wrong\n  For reason: {}", why);
				return null_mut()
			}
		};
		let c_msg: &CStr = CStr::from_ptr(msg.as_ptr());
		let recv_msg: Vec<u8> = match c_msg.to_bytes().from_base64()
		{
			Ok(r) => r,
			Err(_) =>
			{
				println!("Invalid Base64 string");
				return null_mut()
			}
		};

		let dec_dat: Vec<u8> = match decrypt_buf(&c_pri_key, &recv_msg)
		{
			Ok(d) => d,
			Err(_) =>
			{
				println!("Couldn't decrypt data");
				return null_mut()
			}
		};

		let result = match std::str::from_utf8(&dec_dat)
		{
			Ok(v) => v,
			Err(_) =>
			{
				println!("Invalid UTF-8");
				return null_mut()
			}
    };

		let output = env.new_string(result).expect("Couldn't create java string!");
		output.into_inner()
	}
}

// End For JNI
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
// Begin For Curve25519

pub enum EncryptError
{
	RngInitializationFailed
}

pub fn encrypt_buf(public_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, EncryptError>
{
	let mut ephemeral_secret_key = [0u8; 32];
	match thread_rng().try_fill(&mut ephemeral_secret_key[..])
	{
		Ok(eph) => eph,
		Err(_) => return Err(EncryptError::RngInitializationFailed),
	}

	let ephemeral_public_key: [u8; 32] = curve25519_base(&ephemeral_secret_key[..]);
	let symmetric_key = curve25519(&ephemeral_secret_key[..], &public_key[..]);

	let mut c = ChaCha20Poly1305::new(&symmetric_key, &[0u8; 8][..], &[]);

	let mut output = vec![0; 32 + 16 + message.len()];
	let mut tag = [0u8; 16];
	c.encrypt(message, &mut output[32+16..], &mut tag[..]);

	for (dest, src) in (&mut output[0..32]).iter_mut().zip(ephemeral_public_key.iter())
	{
		*dest = *src;
	}

	for (dest, src) in (&mut output[32..48]).iter_mut().zip(tag.iter())
	{
		*dest = *src;
	}

	Ok(output)
}

pub enum DecryptError
{
	Malformed,
	Invalid
}

pub fn decrypt_buf(private_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, DecryptError>
{
	if message.len() < 48
	{
		return Err(DecryptError::Malformed);
	}

	let ephemeral_public_key = &message[0..32];
	let tag = &message[32..48];
	let ciphertext = &message[48..];

	let mut plaintext = vec![0; ciphertext.len()];
	let symmetric_key = curve25519(private_key, ephemeral_public_key);

	let mut decrypter = ChaCha20Poly1305::new(&symmetric_key[..], &[0u8; 8][..], &[]);
	if !decrypter.decrypt(ciphertext, &mut plaintext[..], tag)
	{
		return Err(DecryptError::Invalid);
	}

	Ok(plaintext)
}

// End For Curve25519
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests
{
	#[test]
	fn it_works() {}
}
