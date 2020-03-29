//
//  ViewController.swift
//  CryptoDemo
//
//  Created by Tung Thanh Nguyen on 6/17/17.
//  Copyright © 2017 Comtasoft. All rights reserved.
//

import UIKit

class ViewController: UIViewController
{
	@IBOutlet var message: UITextField!
	@IBOutlet var lblEncryptedText: UILabel!
	@IBOutlet var lblDecryptedText: UILabel!

	let crypto = RustCrypto()!

	let pubKey = "ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic="
	let priKey = "E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8="

	override func viewDidLoad()
	{
		super.viewDidLoad()
		// Do any additional setup after loading the view, typically from a nib.
	}

	override func didReceiveMemoryWarning()
	{
		super.didReceiveMemoryWarning()
		// Dispose of any resources that can be recreated.
	}

	@IBAction func action(_ sender: UIButton)
	{
		if !(message.text?.isEmpty)! && !(message.text?.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines).isEmpty)!
		{
			let c = crypto.encrypt(pubKey, rawMessage: message.text?.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines))
			let d = crypto.decrypt(priKey, encryptedMessage: c)

			lblEncryptedText.text = c
			lblDecryptedText.text = d
		}
	}
}
