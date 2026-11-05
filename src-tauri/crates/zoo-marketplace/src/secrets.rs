//! Where module credentials are kept. The app uses the OS keychain; tests use memory.

use std::collections::HashMap;
use std::sync::Mutex;

use zoo_module_client::Secret;

/// Accounts are `<id>@<marketplace>/<key>`.
pub trait SecretStore: Send + Sync {
    fn get(&self, account: &str) -> Result<Option<Secret>, String>;
    fn set(&self, account: &str, value: &Secret) -> Result<(), String>;
    /// Deleting a missing account is not an error.
    fn delete(&self, account: &str) -> Result<(), String>;
}

#[derive(Default)]
pub struct MemorySecrets(Mutex<HashMap<String, Secret>>);

impl SecretStore for MemorySecrets {
    fn get(&self, account: &str) -> Result<Option<Secret>, String> {
        Ok(self.0.lock().unwrap().get(account).cloned())
    }
    fn set(&self, account: &str, value: &Secret) -> Result<(), String> {
        self.0.lock().unwrap().insert(account.into(), value.clone());
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<(), String> {
        self.0.lock().unwrap().remove(account);
        Ok(())
    }
}
