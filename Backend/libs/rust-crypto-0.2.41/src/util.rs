// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern
{
	pub fn rust_crypto_util_supports_aesni() -> u32;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn supports_aesni() -> bool
{
	unsafe
	{
		rust_crypto_util_supports_aesni() != 0
	}
	// false
}

extern
{
	pub fn rust_crypto_util_fixed_time_eq_asm(lhsp: *const u8, rhsp: *const u8, count: usize) -> u32;
	pub fn rust_crypto_util_secure_memset(dst: *mut u8, val: u8, count: usize);
}

pub fn secure_memset(dst: &mut [u8], val: u8)
{
	unsafe
	{
		rust_crypto_util_secure_memset(dst.as_mut_ptr(), val, dst.len() as usize);
	}
}

/// Compare two vectors using a fixed number of operations. If the two vectors are not of equal
/// length, the function returns false immediately.
pub fn fixed_time_eq(lhs: &[u8], rhs: &[u8]) -> bool
{
	if lhs.len() != rhs.len() { false }
	else
	{
		// let count = lhs.len() as libc::size_t;
		// unsafe
		// {
		// 	let lhsp = lhs.get_unchecked(0);
		// 	let rhsp = rhs.get_unchecked(0);
		// 	rust_crypto_util_fixed_time_eq_asm(lhsp, rhsp, count) == 0
		// }
		// Replace with...
		lhs.iter().zip(rhs).all(|(a, b)| a == b)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct OwnedIv
{
	base_iv: [u8; 16],
	iv: [u8; 16],
	rev: bool, // ge or le bytes
	re: bool, // round enough
	mm: bool // min or max
}

impl OwnedIv
{
	pub fn new(iv: &[u8; 16]) -> OwnedIv
	{
		let mut tmp_base_iv: [u8; 16] = [0; 16];
		let mut tmp_iv: [u8; 16] = [0; 16];
		tmp_base_iv.clone_from_slice(iv);
		tmp_iv.clone_from_slice(iv);
		OwnedIv { base_iv: tmp_base_iv, iv: tmp_iv, rev: false, re: false, mm: true }
	}

	fn do_magic(&mut self) -> u128
	{
		// Reset iv
		self.iv.clone_from_slice(&self.base_iv);

		if !self.rev
		{
			self.rev = !self.rev
		}
		else
		{
			if !self.re
			{
				self.re = !self.re;
				self.rev = !self.rev;
				self.mm = !self.mm
			}
			else
			{
				self.base_iv.reverse();
				self.iv.reverse();
				self.rev = !self.rev;
				self.re = false;
				self.mm = !self.mm
			}
		}

		if !self.rev { u128::from_le_bytes(self.iv) }
		else { u128::from_be_bytes(self.iv) }
	}

	pub fn next_iv(&mut self) -> &[u8; 16]
	{
		let mut tmp: u128;
		if self.rev == false { tmp = u128::from_le_bytes(self.iv) }
		else { tmp = u128::from_be_bytes(self.iv) }

		if self.mm { if tmp == u128::MAX - 1 { tmp = self.do_magic() } }
		else { if tmp == u128::MIN + 1 { tmp = self.do_magic() } }

		if self.mm { tmp += 1 }
		else { tmp -= 1 }

		if !self.rev { self.iv = tmp.to_le_bytes() }
		else { self.iv = tmp.to_be_bytes() }

		&self.iv
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use util::fixed_time_eq;

    #[test]
    pub fn test_fixed_time_eq() {
        let a = [0, 1, 2];
        let b = [0, 1, 2];
        let c = [0, 1, 9];
        let d = [9, 1, 2];
        let e = [2, 1, 0];
        let f = [2, 2, 2];
        let g = [0, 0, 0];

        assert!(fixed_time_eq(&a, &a));
        assert!(fixed_time_eq(&a, &b));

        assert!(!fixed_time_eq(&a, &c));
        assert!(!fixed_time_eq(&a, &d));
        assert!(!fixed_time_eq(&a, &e));
        assert!(!fixed_time_eq(&a, &f));
        assert!(!fixed_time_eq(&a, &g));
    }
}
