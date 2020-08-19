#ifndef __CRYPTO_DEMO_FRAME_H_
#define __CRYPTO_DEMO_FRAME_H_

#include <wx/wxprec.h>
#ifndef WX_PRECOMP
	#include <wx/wx.h>
#endif

class CCryptoDemoApp : public wxApp
{
public:
	CCryptoDemoApp() {}
	virtual ~CCryptoDemoApp() {}

	virtual bool OnInit();
};

#endif
