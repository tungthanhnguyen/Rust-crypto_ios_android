//
//  RustCrypto.m
//  CryptoDemo
//
//  Created by Tung Thanh Nguyen on 6/17/17.
//  Copyright © 2017 Comtasoft. All rights reserved.
//

#import "RustCrypto.h"

@implementation RustCrypto

- (instancetype) init
{
	self = [super init];
	return self;
}

- (NSString*) encrypt:(NSString*) pubKey rawMessage:(NSString*) message
{
	const char* result = (const char *) rust_encrypt([pubKey UTF8String], [message UTF8String]);
	return [[NSString alloc] initWithCString:result encoding:NSUTF8StringEncoding];
}

- (NSString*) decrypt:(NSString*) priKey encryptedMessage:(NSString*) message
{
	const char* result = rust_decrypt([priKey UTF8String], [message UTF8String]);
	return [[NSString alloc] initWithCString:result encoding:NSUTF8StringEncoding];
}

@end
