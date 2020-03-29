#ifndef __RUST_CRYPTO_H_
#define __RUST_CRYPTO_H_

#if __cplusplus
extern "C"
{
#endif

	const char* rust_encrypt(const char* pub_key, const char* msg);
	const char* rust_decrypt(const char* pri_key, const char* msg);

#if __cplusplus
}
#endif

#endif /* __RUST_CRYPTO_H_ */
