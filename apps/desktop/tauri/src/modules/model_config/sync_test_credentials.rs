//! Test-only, process-local replacement. Never instantiate a platform keychain
//! entry from the saved-content/file-transfer regression suite.
use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi};
use std::{
    any::Any,
    collections::BTreeMap,
    sync::{Arc, Mutex, Once},
};
type Values = Arc<Mutex<BTreeMap<(String, String), Vec<u8>>>>;
#[derive(Default)]
struct Builder(Values);
struct Entry {
    values: Values,
    identity: (String, String),
}
impl CredentialBuilderApi for Builder {
    fn build(
        &self,
        _: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<Credential>> {
        Ok(Box::new(Entry {
            values: self.0.clone(),
            identity: (service.into(), user.into()),
        }))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl CredentialApi for Entry {
    fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
        self.values
            .lock()
            .unwrap()
            .insert(self.identity.clone(), value.to_vec());
        Ok(())
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.values
            .lock()
            .unwrap()
            .get(&self.identity)
            .cloned()
            .ok_or(keyring::Error::NoEntry)
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        self.values
            .lock()
            .unwrap()
            .remove(&self.identity)
            .map(|_| ())
            .ok_or(keyring::Error::NoEntry)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
pub fn install() {
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(|| keyring::set_default_credential_builder(Box::<Builder>::default()));
}
