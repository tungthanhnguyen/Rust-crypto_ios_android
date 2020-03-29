//
//  RustCrypto.h
//  CryptoDemo
//
//  Created by Tung Thanh Nguyen on 6/17/17.
//  Copyright © 2017 Comtasoft. All rights reserved.
//

#import <Foundation/Foundation.h>

#include "rust_crypto.h"

@interface RustCrypto : NSObject

- (instancetype) init;

- (NSString*) encrypt:(NSString*) pubKey rawMessage:(NSString*) message;
- (NSString*) decrypt:(NSString*) priKey encryptedMessage:(NSString*) message;

@end
