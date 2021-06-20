extern crate rust_crypto;
extern crate rustc_serialize;
extern crate web_sys;

mod utils;

use std::vec::Vec;

use wasm_bindgen::prelude::*;

use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};

use rust_crypto::{encrypt_buf, decrypt_buf};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log
{
	( $( $t:tt )* ) =>
	{
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

#[wasm_bindgen(js_name = "encrypt")]
pub fn wasm_encrypt(pub_key: String, msg: String) -> String
{
	// utils::set_panic_hook();

	let mut arr = [0u8; 32]; // temp buffer
	let public_key: &[u8; 32] = match pub_key.trim().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				// let e_str = format!("Public key size is wrong");
				// log!("{}", e_str);
				return String::new()
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			// let e_str = format!("Public key is wrong\n  For reason: {}", why);
			// log!("{}", e_str);
			return String::new()
		}
	};

	let enc_dat: Vec<u8> = match encrypt_buf(&public_key, msg.as_bytes())
	{
		Ok(d) => d,
		Err(_) =>
		{
			// let e_str = format!("Couldn't encrypt a buffer");
			// log!("{}", e_str);
			return String::new()
		}
	};

	String::from(enc_dat.as_slice().to_base64(STANDARD))
}

#[wasm_bindgen(js_name = "decrypt")]
pub fn wasm_decrypt(pri_key: String, msg: String) -> String
{
	// utils::set_panic_hook();

	let mut arr = [0u8; 32]; // temp buff
	let private_key: &[u8; 32] = match pri_key.trim().from_base64()
	{
		Ok(m) =>
		{
			if m.len() != 32
			{
				// let e_str = format!("Private key size is wrong");
				// log!("{}", e_str);
				return String::new()
			}
			arr.copy_from_slice(&m);
			&arr
		},
		Err(why) =>
		{
			// let e_str = format!("Private key is wrong\n  For reason: {}", why);
			// log!("{}", e_str);
			return String::new()
		}
	};

	let recv_msg: Vec<u8> = match msg.as_bytes().from_base64()
	{
		Ok(r) => r,
		Err(_) =>
		{
			// let e_str = format!("Invalid Base64 string");
			// log!("{}", e_str);
			return String::new()
		}
	};

	let dec_dat: Vec<u8> = match decrypt_buf(&private_key, &recv_msg)
	{
		Ok(d) => d,
		Err(_) =>
		{
			// let e_str = format!("Couldn't decrypt data");
			// log!("{}", e_str);
			return String::new()
		}
	};

	String::from_utf8(dec_dat).unwrap()
}
