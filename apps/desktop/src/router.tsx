import { Navigate, Route, Routes } from "react-router-dom";
import { MainLayout } from "@/components/layout/MainLayout";
import { useWallet } from "@/context/WalletContext";
import { ActivityPage } from "@/pages/ActivityPage";
import { DashboardPage } from "@/pages/DashboardPage";
import { ManageAccountsPage } from "@/pages/ManageAccountsPage";
import { ReceivePage } from "@/pages/ReceivePage";
import { SendPage } from "@/pages/SendPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { SwapPage } from "@/pages/SwapPage";
import { UnlockPage } from "@/pages/UnlockPage";
import { ConfirmSeedPage } from "@/pages/onboarding/ConfirmSeedPage";
import { CreateImportPage } from "@/pages/onboarding/CreateImportPage";
import { ImportBackupPage } from "@/pages/onboarding/ImportBackupPage";
import { SetPasswordPage } from "@/pages/onboarding/SetPasswordPage";
import { ShowSeedPage } from "@/pages/onboarding/ShowSeedPage";
import { WalletReadyPage } from "@/pages/onboarding/WalletReadyPage";
import { WelcomePage } from "@/pages/onboarding/WelcomePage";

export function AppRouter() {
  const { loading, walletExists, unlocked } = useWallet();

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center text-muted-foreground">
        Loading Taurvia...
      </div>
    );
  }

  return (
    <Routes>
      {!walletExists && (
        <>
          <Route path="/onboarding" element={<WelcomePage />} />
          <Route path="/onboarding/create" element={<CreateImportPage mode="create" />} />
          <Route path="/onboarding/import" element={<CreateImportPage mode="import" />} />
          <Route path="/onboarding/import-backup" element={<ImportBackupPage />} />
          <Route path="/onboarding/seed" element={<ShowSeedPage />} />
          <Route path="/onboarding/confirm" element={<ConfirmSeedPage />} />
          <Route path="/onboarding/password" element={<SetPasswordPage />} />
          <Route path="/onboarding/ready" element={<WalletReadyPage />} />
          <Route path="*" element={<Navigate to="/onboarding" replace />} />
        </>
      )}

      {walletExists && !unlocked && (
        <>
          <Route path="/unlock" element={<UnlockPage />} />
          <Route path="*" element={<Navigate to="/unlock" replace />} />
        </>
      )}

      {walletExists && unlocked && (
        <Route element={<MainLayout />}>
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/swap" element={<SwapPage />} />
          <Route path="/send" element={<SendPage />} />
          <Route path="/receive" element={<ReceivePage />} />
          <Route path="/activity" element={<ActivityPage />} />
          <Route path="/accounts" element={<ManageAccountsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/settings/:section" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Route>
      )}
    </Routes>
  );
}
