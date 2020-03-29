package com.comtasoft.cryptodemo;

import android.app.Activity;
import android.os.Bundle;
import android.view.View;
import android.widget.EditText;
import android.widget.TextView;

public class MainActivity extends Activity
{
	private String pubKey = "ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic=";
	private String priKey = "E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8=";

	static { System.loadLibrary("rust_crypto"); }

	private static native String RustEncrypt(final String pubKey, final String message);
	private static native String RustDecrypt(final String priKey, final String message);

	@Override
	protected void onCreate(Bundle savedInstanceState)
	{
		super.onCreate(savedInstanceState);
		setContentView(R.layout.activity_main);
	}

	public void OnAction(View view)
	{
		String sMess = ((EditText) findViewById(R.id.edtMess)).getText().toString();
		if (!sMess.isEmpty() && !sMess.trim().isEmpty())
		{
			String encMess = RustEncrypt(pubKey, sMess);
			String decMess = RustDecrypt(priKey, encMess);

			((TextView) findViewById(R.id.lblEncryptedText)).setText(encMess);
			((TextView) findViewById(R.id.lblDecryptedText)).setText(decMess);
		}
	}
}
