//! Authorization and delegation through the host service handle API.

use std::time::{Duration, Instant};

use obscur_plugin_host::capability::{
    CapabilityAccessError, CapabilityIssuer, CapabilityRevoker, ServiceScope,
};
use obscur_plugin_model::{
    identity::{PluginId, PluginPrincipal},
    manifest::Version,
};

fn principal(installation: &str) -> PluginPrincipal {
    PluginPrincipal::new(PluginId::new("example").unwrap(), installation)
}

fn scope(operations: &[&str]) -> ServiceScope {
    ServiceScope::new(
        "resources",
        Version::new(1, 0, 0),
        operations.iter().copied(),
    )
}

#[test]
fn delegates_only_narrower_scopes_and_revokes_descendants() {
    let mut issuer = CapabilityIssuer::default();
    let provider = principal("provider");
    let consumer = principal("consumer");
    let deadline = Instant::now() + Duration::from_secs(60);
    let root = issuer
        .issue(provider.clone(), scope(&["read", "list"]), Some(deadline))
        .unwrap();
    let child = issuer
        .delegate(&root, consumer.clone(), scope(&["read"]), None)
        .unwrap();
    let grandchild = issuer
        .delegate(&child, provider.clone(), scope(&["read"]), None)
        .unwrap();
    let independent = issuer
        .issue(consumer.clone(), scope(&["list"]), None)
        .unwrap();
    let version = Version::new(1, 0, 0);
    assert_eq!(
        issuer.active_grants().collect::<Vec<_>>(),
        [
            (&provider, &scope(&["read", "list"]), Some(deadline)),
            (&consumer, &scope(&["read"]), Some(deadline)),
            (&provider, &scope(&["read"]), Some(deadline)),
            (&consumer, &scope(&["list"]), None),
        ]
    );

    assert!(
        issuer
            .authorize(&child, &consumer, "resources", &version, "read")
            .is_ok()
    );
    assert_eq!(
        issuer.authorize(&child, &provider, "resources", &version, "read"),
        Err(CapabilityAccessError::Denied)
    );
    assert!(
        issuer
            .authorize(&child, &consumer, "resources", &version, "list")
            .is_err()
    );
    assert!(
        issuer
            .authorize(&child, &consumer, "other", &version, "read")
            .is_err()
    );
    assert!(
        issuer
            .authorize(
                &child,
                &consumer,
                "resources",
                &Version::new(2, 0, 0),
                "read"
            )
            .is_err()
    );
    assert!(
        issuer
            .delegate(&child, provider.clone(), scope(&["list"]), None)
            .is_err()
    );
    assert!(
        issuer
            .delegate(
                &root,
                consumer.clone(),
                scope(&["read"]),
                Some(deadline + Duration::from_secs(1))
            )
            .is_err()
    );

    assert_eq!(
        CapabilityRevoker::revoke_principal(&mut issuer, &provider),
        3
    );
    assert_eq!(
        issuer.active_grants().collect::<Vec<_>>(),
        [(&consumer, &scope(&["list"]), None)]
    );
    assert!(
        issuer
            .authorize(&grandchild, &provider, "resources", &version, "read")
            .is_err()
    );
    assert!(
        issuer
            .authorize(&child, &consumer, "resources", &version, "read")
            .is_err()
    );
    assert!(
        issuer
            .authorize(&independent, &consumer, "resources", &version, "list")
            .is_ok()
    );
    assert_eq!(CapabilityRevoker::revoke(&mut issuer, &independent), 1);
    assert_eq!(CapabilityRevoker::revoke(&mut issuer, &independent), 0);
}

#[test]
fn rejects_expired_grants_and_handles_from_other_issuers() {
    let principal = principal("installation");
    let mut issuer = CapabilityIssuer::default();
    let mut foreign = CapabilityIssuer::default();
    let handle = issuer
        .issue(principal.clone(), scope(&["read"]), None)
        .unwrap();
    let _ = foreign
        .issue(principal.clone(), scope(&["read"]), None)
        .unwrap();
    assert!(
        foreign
            .authorize(
                &handle,
                &principal,
                "resources",
                &Version::new(1, 0, 0),
                "read"
            )
            .is_err()
    );
    assert_eq!(CapabilityRevoker::revoke(&mut foreign, &handle), 0);
    assert!(matches!(
        issuer.issue(principal, scope(&["read"]), Some(Instant::now())),
        Err(CapabilityAccessError::Denied)
    ));
}
