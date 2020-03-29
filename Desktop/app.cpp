#include "app.h"
#include "MainFrame.h"

wxIMPLEMENT_APP(CCryptoDemoApp);

bool CCryptoDemoApp::OnInit()
{
	if (!wxApp::OnInit()) return false;

	// Create the main application window
	CMainFrame *pFrame = new CMainFrame(NULL, wxID_ANY, wxT("Test rust-crypto with wxWidgets"));

	// Show it
	pFrame->Show(true);

	// Start the event loop
	return true;
}
