package com.comtasoft.cryptodemo

import android.os.Bundle
import android.view.View
import android.widget.EditText
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity()
{
	private val pubKey = "ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic="
	private val priKey = "E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8="

	init
	{
		System.loadLibrary("rust_crypto")
	}

	private external fun RustEncrypt(pubKey: String, message: String): String?
	private external fun RustDecrypt(priKey: String, message: String): String?

	private lateinit var edtMess: EditText
	private lateinit var lblEncryptedText: TextView
	private lateinit var lblDecryptedText: TextView

	override fun onCreate(savedInstanceState: Bundle?)
	{
		super.onCreate(savedInstanceState)
		setContentView(R.layout.activity_main)

		edtMess = findViewById(R.id.edtMess)
		lblEncryptedText = findViewById(R.id.lblEncryptedText)
		lblDecryptedText = findViewById(R.id.lblDecryptedText)
	}

	fun onAction(view: View)
	{
		val sMess = edtMess.text.toString().trim()
		if (sMess.isNotEmpty())
		{
			val encMess = RustEncrypt(pubKey, sMess)
			val decMess = RustDecrypt(priKey, encMess.toString())

			lblEncryptedText.text = encMess
			lblDecryptedText.text = decMess
		}
	}
}