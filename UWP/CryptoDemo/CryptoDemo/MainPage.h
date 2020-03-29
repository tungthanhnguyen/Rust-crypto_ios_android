#pragma once

#include "MainPage.g.h"

#include <string_view>

using namespace std;
using namespace winrt;

namespace winrt::CryptoDemo::implementation
{
	struct MainPage : MainPageT<MainPage>
	{
		wstring m_pubKey, m_priKey;

		MainPage();

		int32_t MyProperty();
		void MyProperty(int32_t value);

		void BtnAction_Click(Windows::Foundation::IInspectable const& sender, Windows::UI::Xaml::RoutedEventArgs const& args);
	};
}

namespace winrt::CryptoDemo::factory_implementation
{
  struct MainPage : MainPageT<MainPage, implementation::MainPage>
  {
  };
}
