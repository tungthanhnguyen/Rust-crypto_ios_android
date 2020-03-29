#ifndef __MAIN_FRAME_H_
#define __MAIN_FRAME_H_

#include <wx/wxprec.h>
#ifndef WX_PRECOMP
	#include <wx/wx.h>
#endif

#include <wx/app.h>

class CMainFrame : public wxFrame
{
	enum
	{
		ID_Action = wxID_HIGHEST + 1,
		ID_Encrypted_Text,
		ID_Decrypted_Text
	};

public:
	CMainFrame(wxWindow* parent, wxWindowID id, const wxString& title,
		const wxPoint& pos = wxDefaultPosition, const wxSize& size = wxDefaultSize,
		long style = wxDEFAULT_FRAME_STYLE | wxSUNKEN_BORDER);
	~CMainFrame();

protected:
	void OnAction(wxCommandEvent& evt);
	void OnQuit(wxCommandEvent& evt);

private:
	wxTextCtrl* m_pTxtInputText;
	wxButton* m_pBtnAction;
	wxTextCtrl* m_pEncryptedText;
	wxStaticText* m_pDecryptedText;

	wxString m_pubKey;
	wxString m_priKey;

	wxDECLARE_EVENT_TABLE();
};

#endif
