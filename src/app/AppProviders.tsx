import type { ReactNode } from "react";
import { DocumentHistoryProvider, DocumentWorkspaceProvider } from "../features/documents";
import { MarkdownSearchProvider, MarkdownWorkspaceProvider } from "../features/markdown";
import { NotificationProvider } from "../features/notifications";
import { PdfSearchProvider, PdfWorkspaceProvider } from "../features/pdf";
import { SettingsProvider } from "../features/settings";

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <NotificationProvider>
      <SettingsProvider>
        <DocumentHistoryProvider>
          <DocumentWorkspaceProvider>
            <PdfWorkspaceProvider>
              <PdfSearchProvider>
                <MarkdownWorkspaceProvider>
                  <MarkdownSearchProvider>{children}</MarkdownSearchProvider>
                </MarkdownWorkspaceProvider>
              </PdfSearchProvider>
            </PdfWorkspaceProvider>
          </DocumentWorkspaceProvider>
        </DocumentHistoryProvider>
      </SettingsProvider>
    </NotificationProvider>
  );
}
