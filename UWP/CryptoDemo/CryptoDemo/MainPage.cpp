#include "pch.h"
#include "MainPage.h"
#include "MainPage.g.cpp"

using namespace std;
using namespace winrt;
using namespace Windows::UI::Xaml;

#include "..\..\..\Backend\rust_crypto\src\rust_crypto.h"

std::string make_string(const std::wstring& wstring)
{
	auto wideData = wstring.c_str();
	int bufferSize = WideCharToMultiByte(CP_UTF8, 0, wideData, -1, nullptr, 0, NULL, NULL);
	auto utf8 = std::make_unique<char[]>(bufferSize);
	if (0 == WideCharToMultiByte(CP_UTF8, 0, wideData, -1, utf8.get(), bufferSize, NULL, NULL))
		throw std::exception("Can't convert string to UTF8");

	return std::string(utf8.get());
}

std::wstring make_wstring(const std::string& string)
{
	auto utf8Data = string.c_str();
	int bufferSize = MultiByteToWideChar(CP_UTF8, 0, utf8Data, -1, nullptr, 0);
	auto wide = std::make_unique<wchar_t[]>(bufferSize);
	if (0 == MultiByteToWideChar(CP_UTF8, 0, utf8Data, -1, wide.get(), bufferSize))
		throw std::exception("Can't convert string to Unicode");

	return std::wstring(wide.get());
}

winrt::hstring trim_string(const winrt::hstring& string)
{
	if (!string.empty())
	{
		//convert to wstring
		std::wstring classicString(string.data());

		//trimming
		classicString.erase(0, classicString.find_first_not_of(' '));
		classicString.erase(classicString.find_last_not_of(' ') + 1);

		//convert to Platform::String
		return winrt::hstring(classicString.c_str());
	}
	return string;
}

namespace winrt::CryptoDemo::implementation
{
	MainPage::MainPage()
	{
		InitializeComponent();

		m_pubKey = L"ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic=";
		m_priKey = L"E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8=";
	}

	int32_t MainPage::MyProperty()
	{
		throw hresult_not_implemented();
	}

	void MainPage::MyProperty(int32_t /* value */)
	{
		throw hresult_not_implemented();
	}

	void MainPage::BtnAction_Click(IInspectable const& /* sender */, RoutedEventArgs const& /* args */)
	{
		hstring n_inputText = trim_string(txtInputText().Text());
		if (!n_inputText.empty())
		{
			wstring ws_inputText{ n_inputText };

			string s_pubKey = make_string(m_pubKey);
			string s_priKey = make_string(m_priKey);
			string s_inputText = make_string(ws_inputText);

			string s_encryptedText{ rust_encrypt(s_pubKey.c_str(), s_inputText.c_str()) };
			string s_decryptedText{ rust_decrypt(s_priKey.c_str(), s_encryptedText.c_str()) };

			wstring n_encryptedText = make_wstring(s_encryptedText);
			wstring n_decryptedText = make_wstring(s_decryptedText);

			txtEncryptedText().Text(n_encryptedText);
			txtDecryptedText().Text(n_decryptedText);
		}
		else
		{
			txtEncryptedText().Text(L"");
			txtDecryptedText().Text(L"");
		}
	}
}
