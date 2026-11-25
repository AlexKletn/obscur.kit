//! Secure storage and lease authorization through the host API.

use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    time::{Duration, Instant},
};

use obscur_plugin_host::storage::{CredentialScope, SecureStorageBackend, SecureStorageService};
use obscur_plugin_model::identity::{PluginId, PluginPrincipal};

#[derive(Default)]
struct TestKeyring {
    secrets: RefCell<HashMap<(PluginPrincipal, String), Vec<u8>>>,
}

impl SecureStorageBackend for TestKeyring {
    fn load(&self, principal: &PluginPrincipal, key: &str) -> io::Result<Option<Vec<u8>>> {
        Ok(self
            .secrets
            .borrow()
            .get(&(principal.clone(), key.into()))
            .cloned())
    }

    fn store(&self, principal: &PluginPrincipal, key: &str, value: &[u8]) -> io::Result<()> {
        self.secrets
            .borrow_mut()
            .insert((principal.clone(), key.into()), value.to_vec());
        Ok(())
    }

    fn remove(&self, principal: &PluginPrincipal, key: &str) -> io::Result<bool> {
        Ok(self
            .secrets
            .borrow_mut()
            .remove(&(principal.clone(), key.into()))
            .is_some())
    }
}

#[test]
fn leases_bind_to_principal_scope_and_issuer_and_can_be_revoked() {
    let service = SecureStorageService::new(TestKeyring::default());
    let foreign = SecureStorageService::new(TestKeyring::default());
    let id = PluginId::new("example").unwrap();
    let principal = PluginPrincipal::new(id.clone(), "installation-a");
    let other = PluginPrincipal::new(id, "installation-b");
    let scope = CredentialScope::new("network", "fetch", "example.com");
    let expires = Instant::now() + Duration::from_secs(60);
    let key = "private-key-name";
    service.store(&principal, key, b"credential").unwrap();
    service.store(&other, key, b"other").unwrap();
    let lease = service.issue(principal.clone(), key, scope.clone(), expires);

    assert_eq!(
        service
            .with_secret(&lease, &principal, &scope, |bytes| bytes == b"credential")
            .unwrap(),
        Some(true)
    );
    let other_scope = CredentialScope::new("network", "fetch", "other.example.com");
    for (issuer, caller, scope) in [
        (&service, &other, &scope),
        (&service, &principal, &other_scope),
        (&foreign, &principal, &scope),
    ] {
        let error = issuer
            .with_secret(&lease, caller, scope, |_| panic!("denied operation ran"))
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    assert!(!foreign.revoke(&lease));
    let copy = lease.clone();
    assert!(service.revoke(&lease));
    assert!(
        service
            .with_secret(&copy, &principal, &scope, |_| ())
            .is_err()
    );
    assert!(!format!("{lease:?}").contains(key));

    let expired = service.issue(principal.clone(), key, scope.clone(), Instant::now());
    assert!(
        service
            .with_secret(&expired, &principal, &scope, |_| ())
            .is_err()
    );
    let live = service.issue(principal.clone(), key, scope.clone(), expires);
    assert!(service.remove(&principal, key).unwrap());
    assert_eq!(
        service
            .with_secret(&live, &principal, &scope, |_| ())
            .unwrap(),
        None
    );
    let other_lease = service.issue(other.clone(), key, scope.clone(), expires);
    assert_eq!(
        service
            .with_secret(&other_lease, &other, &scope, |bytes| bytes == b"other")
            .unwrap(),
        Some(true)
    );
}
