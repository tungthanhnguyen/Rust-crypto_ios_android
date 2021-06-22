#include "MainFrame.h"

#include "../Backend/rust_crypto/src/rust_crypto.h"

wxBEGIN_EVENT_TABLE(CMainFrame, wxFrame)
	EVT_MENU(wxID_EXIT, CMainFrame::OnQuit)
	EVT_BUTTON(ID_Action, CMainFrame::OnAction)
wxEND_EVENT_TABLE()

CMainFrame::CMainFrame(wxWindow* parent, wxWindowID id, const wxString& title, const wxPoint& pos, const wxSize& size, long style)
	: wxFrame(parent, id, title, pos, size, style)
{
	m_pubKey = wxString("ij2bL9WP9+R27/8VjjxVWoba4o3IbTpOme0o28Hjwic=");
	m_priKey = wxString("E3B3Q3TglEQKg+w7CBkQ8XmrqemJ4fYGkTzWPSMUhW8=");

	wxMenu* n_pMenuApp = new wxMenu;
	n_pMenuApp->Append(wxID_EXIT, wxT("Exit"));

	wxMenuBar* n_pMainMenuBar = new wxMenuBar;
	n_pMainMenuBar->Append(n_pMenuApp, wxT("App"));
	SetMenuBar(n_pMainMenuBar);

	m_pTxtInputText = new wxTextCtrl(this, wxID_ANY, wxT(""));
	m_pBtnAction = new wxButton(this, ID_Action, wxT("Action"));
	m_pEncryptedText = new wxTextCtrl(this, ID_Encrypted_Text, wxT(""), wxDefaultPosition, wxDefaultSize, wxALIGN_LEFT | wxTE_READONLY | wxTE_MULTILINE | wxTE_CHARWRAP);
	m_pDecryptedText = new wxStaticText(this, ID_Decrypted_Text, wxT(""));

	wxBoxSizer* n_pSizer = new wxBoxSizer(wxVERTICAL);
	if (n_pSizer != NULL)
	{
		wxFlexGridSizer* n_pMainSizer = new wxFlexGridSizer(4, 2, 5, 5);
		if (n_pMainSizer != NULL)
		{
			n_pMainSizer->AddGrowableRow(2, 1);
			n_pMainSizer->AddGrowableCol(1, 1);

			n_pMainSizer->Add(new wxStaticText(this, wxID_ANY, wxT("Input Text:")), 1,  wxALIGN_RIGHT | wxALIGN_CENTER_VERTICAL);
			n_pMainSizer->Add(m_pTxtInputText, 1, wxEXPAND);

			n_pMainSizer->Add(m_pBtnAction, wxALIGN_RIGHT);
			n_pMainSizer->AddStretchSpacer();

			n_pMainSizer->Add(new wxStaticText(this, wxID_ANY, wxT("Encrypted Text:")), 1, wxALIGN_RIGHT | wxALIGN_CENTER_VERTICAL | wxEXPAND);
			n_pMainSizer->Add(m_pEncryptedText, 1, wxEXPAND);

			n_pMainSizer->Add(new wxStaticText(this, wxID_ANY, wxT("Decrypted Text:")), 1, wxALIGN_RIGHT | wxALIGN_CENTER_VERTICAL);
			n_pMainSizer->Add(m_pDecryptedText, 1, wxEXPAND);

			n_pSizer->Add(n_pMainSizer, 1, wxALL | wxEXPAND, 10);

			SetSizer(n_pSizer);
			n_pSizer->SetSizeHints(this);
		}
	}

#ifdef __WXGTK__
	SetSize(wxSize(480, 300));
	SetMinSize(wxSize(480, 300));
#else
	SetSize(wxSize(480, GetBestHeight(480)));
	SetMinSize(wxSize(480, GetBestHeight(480)));
#endif

	Centre(wxBOTH | wxCENTRE_ON_SCREEN);
}

CMainFrame::~CMainFrame()
{
}

void CMainFrame::OnAction(wxCommandEvent& WXUNUSED(evt))
{
	wxString n_txtMessage = m_pTxtInputText->GetValue().Trim(false).Trim();
	if (!n_txtMessage.IsEmpty())
	{
		wxString n_txtEncryptedText(rust_encrypt((const char*) m_pubKey.mb_str(wxConvUTF8), (const char*) n_txtMessage.mb_str(wxConvUTF8)), wxConvUTF8);
		wxString n_txtDecryptedText(rust_decrypt((const char*) m_priKey.mb_str(wxConvUTF8), (const char*) n_txtEncryptedText.mb_str(wxConvUTF8)), wxConvUTF8);

		m_pEncryptedText->SetValue(n_txtEncryptedText);
		m_pDecryptedText->SetLabel(n_txtDecryptedText);
	}
	else
	{
		m_pEncryptedText->SetValue(wxT(""));
		m_pDecryptedText->SetLabel(wxT(""));
	}
}

void CMainFrame::OnQuit(wxCommandEvent& WXUNUSED(evt))
{
	Close(true);
}
