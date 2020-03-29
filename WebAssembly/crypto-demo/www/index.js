import * as wasm from "crypto-demo";

var pub_key = "ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic=";
var pri_key = "E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8=";

function doAction()
{
	// debugger;

	var s_input_text = document.getElementById("f_input_text").value.trim();
	if (s_input_text)
	{
		var s_encrypted_text = wasm.encrypt(pub_key, s_input_text);
		var s_decrypted_text = wasm.decrypt(pri_key, s_encrypted_text);

		document.getElementById("l_encrypted_text").innerText = s_encrypted_text;
		document.getElementById("l_decrypted_text").innerText = s_decrypted_text;
	}
	else
	{
		document.getElementById("l_encrypted_text").innerText = null;
		document.getElementById("l_decrypted_text").innerText = null;
	}
}

document.getElementById('b_action').addEventListener('click', doAction);
