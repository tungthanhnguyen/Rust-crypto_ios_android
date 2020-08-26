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
	pos: usize,
	pp: usize, // pinned pos
	rev: bool, // ge or le bytes
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
		let mut sum: usize = 0;
		for item in iv.iter() { sum += *item as usize }
		let pp: usize = sum % 9;
		OwnedIv { base_iv: tmp_base_iv, iv: tmp_iv, pos: pp, pp: pp, rev: false, mm: true }
	}

	fn do_magic(&mut self) -> u64
	{
		// Reset iv
		self.iv[self.pos] = self.base_iv[self.pos];
		self.iv[self.pos + 1] = self.base_iv[self.pos + 1];
		self.iv[self.pos + 2] = self.base_iv[self.pos + 2];
		self.iv[self.pos + 3] = self.base_iv[self.pos + 3];
		self.iv[self.pos + 4] = self.base_iv[self.pos + 4];
		self.iv[self.pos + 5] = self.base_iv[self.pos + 5];
		self.iv[self.pos + 6] = self.base_iv[self.pos + 6];
		self.iv[self.pos + 7] = self.base_iv[self.pos + 7];

		if self.rev == false
		{
			if self.pos > 0
			{
				self.pos -= 1;
				if self.pos == self.pp
				{
					if self.mm == false
					{
						self.base_iv.reverse();
						self.iv.reverse();
					}
					self.mm = !self.mm
				}
			}
			else { self.rev = true }
		}
		else
		{
			if self.pos < 8 { self.pos += 1 }
			else { self.rev = false }
		}

		let mut eight_bytes: [u8; 8] = [0; 8];
		eight_bytes[0] = self.iv[self.pos];
		eight_bytes[1] = self.iv[self.pos + 1];
		eight_bytes[2] = self.iv[self.pos + 2];
		eight_bytes[3] = self.iv[self.pos + 3];
		eight_bytes[4] = self.iv[self.pos + 4];
		eight_bytes[5] = self.iv[self.pos + 5];
		eight_bytes[6] = self.iv[self.pos + 6];
		eight_bytes[7] = self.iv[self.pos + 7];

		if !self.rev { u64::from_le_bytes(eight_bytes) }
		else { u64::from_be_bytes(eight_bytes) }
	}

	pub fn next_iv(&mut self) -> &[u8; 16]
	{
		let mut eight_bytes: [u8; 8] = [0; 8];
		eight_bytes[0] = self.iv[self.pos];
		eight_bytes[1] = self.iv[self.pos + 1];
		eight_bytes[2] = self.iv[self.pos + 2];
		eight_bytes[3] = self.iv[self.pos + 3];
		eight_bytes[4] = self.iv[self.pos + 4];
		eight_bytes[5] = self.iv[self.pos + 5];
		eight_bytes[6] = self.iv[self.pos + 6];
		eight_bytes[7] = self.iv[self.pos + 7];

		let mut tmp: u64;
		if self.rev == false { tmp = u64::from_le_bytes(eight_bytes) }
		else { tmp = u64::from_be_bytes(eight_bytes) }

		if self.mm { if tmp == u64::MAX - 1 { tmp = self.do_magic() } }
		else { if tmp == u64::MIN + 1 { tmp = self.do_magic() } }

		if self.mm { tmp += 1 }
		else { tmp -= 1 }

		if !self.rev { eight_bytes = tmp.to_le_bytes() }
		else { eight_bytes = tmp.to_be_bytes() }

		self.iv[self.pos] = eight_bytes[0];
		self.iv[self.pos + 1] = eight_bytes[1];
		self.iv[self.pos + 2] = eight_bytes[2];
		self.iv[self.pos + 3] = eight_bytes[3];
		self.iv[self.pos + 4] = eight_bytes[4];
		self.iv[self.pos + 5] = eight_bytes[5];
		self.iv[self.pos + 6] = eight_bytes[6];
		self.iv[self.pos + 7] = eight_bytes[7];

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
