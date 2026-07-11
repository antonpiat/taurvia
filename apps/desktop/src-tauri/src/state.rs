use models::OnboardingDraft;
use std::sync::Mutex;
use wallet_core::WalletService;

pub struct AppState {
    pub wallet: WalletService,
    pub onboarding: Mutex<Option<OnboardingDraft>>,
}

impl AppState {
    pub fn new(wallet: WalletService) -> Self {
        Self {
            wallet,
            onboarding: Mutex::new(None),
        }
    }

    pub fn set_onboarding(&self, draft: OnboardingDraft) {
        *self.onboarding.lock().unwrap() = Some(draft);
    }

    pub fn get_onboarding(&self) -> Option<OnboardingDraft> {
        self.onboarding.lock().unwrap().clone()
    }

    pub fn clear_onboarding(&self) {
        *self.onboarding.lock().unwrap() = None;
    }
}
