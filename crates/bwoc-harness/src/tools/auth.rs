//! Tool authentication — credential broker.
//!
//! **P3 component** — Sīla Adinnādāna (non-theft) + Kalyāṇamitta (good
//! companionship / trust).
//!
//! ## Design
//!
//! Tools declare the credentials they need via [`CredentialRequest`].  The
//! broker fetches **scoped** credentials from the OS keyring (or the injected
//! test mock) and injects them into the child process environment **at exec
//! time only**.
//!
//! ### What "at exec time only" means
//!
//! The injected variables are passed directly to
//! [`tokio::process::Command::env`] calls.  They are:
//!
//! - **Never placed in the prompt** — the broker has no access to the
//!   message history.
//! - **Never logged** — the broker never calls `log!`, `eprintln!`, or any
//!   tracing macro with a credential value.
//! - **Scrubbed from telemetry** — [`CredentialBroker::scrubbed_env`] returns
//!   an env map with all credential keys replaced by `"[REDACTED]"` for any
//!   caller that needs to log or record the child env.
//!
//! ### Compose with sandbox env-scrub
//!
//! The broker is intended to run **after** [`crate::sandbox::scrub_env`]:
//!
//! ```text
//! scrub_env()                  // strips all sensitive vars from process env
//!   → broker.inject(requests)  // injects only the scoped vars the tool declared
//!     → Command::envs(injected) // child process receives exactly the declared creds
//! ```
//!
//! This means the child process never inherits ambient credentials; it only
//! receives the minimal set it declared.
//!
//! ### Keyring trait
//!
//! [`CredentialStore`] is the abstraction.  Tests MUST inject
//! [`InMemoryCredentialStore`] — never a real OS keyring.  Any test that
//! needs a real OS keyring is `#[ignore]`d.

use std::collections::HashMap;

use crate::error::HarnessError;

// ---------------------------------------------------------------------------
// CredentialRequest — what a tool declares it needs
// ---------------------------------------------------------------------------

/// A single credential requirement from a tool.
///
/// Tools should use `CredentialRequest::github_token()` or the appropriate
/// constructor rather than constructing this struct directly.
#[derive(Debug, Clone)]
pub struct CredentialRequest {
    /// The environment variable name to inject into the child process.
    pub env_var: &'static str,
    /// The keyring service name (e.g. `"bwoc/github"`, `"bwoc/npm"`).
    pub keyring_service: &'static str,
    /// The keyring account/entry name.
    pub keyring_account: &'static str,
    /// Whether the tool can run (in degraded mode) if the credential is absent.
    /// `false` = the tool must fail if the credential is unavailable.
    pub optional: bool,
}

impl CredentialRequest {
    /// GitHub personal access token / App token.
    pub const fn github_token() -> Self {
        Self {
            env_var: "GITHUB_TOKEN",
            keyring_service: "bwoc/github",
            keyring_account: "token",
            optional: false,
        }
    }

    /// npm publish / registry token.
    pub const fn npm_token() -> Self {
        Self {
            env_var: "NPM_TOKEN",
            keyring_service: "bwoc/npm",
            keyring_account: "token",
            optional: true,
        }
    }

    /// Generic custom credential.
    pub const fn custom(
        env_var: &'static str,
        keyring_service: &'static str,
        keyring_account: &'static str,
        optional: bool,
    ) -> Self {
        Self {
            env_var,
            keyring_service,
            keyring_account,
            optional,
        }
    }
}

// ---------------------------------------------------------------------------
// CredentialStore trait — the injectable keyring abstraction
// ---------------------------------------------------------------------------

/// Abstraction over the OS keyring.
///
/// Production code uses [`OsKeyringStore`].
/// Tests MUST use [`InMemoryCredentialStore`] — never the OS keyring.
pub trait CredentialStore: Send + Sync {
    /// Fetch the credential for `service`/`account`.
    ///
    /// Returns `Ok(None)` if the entry does not exist.
    /// Returns `Err` on a keyring backend error.
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialStoreError {
    #[error("keyring backend error: {0}")]
    Backend(String),
}

// ---------------------------------------------------------------------------
// InMemoryCredentialStore — test double (never requires OS keyring)
// ---------------------------------------------------------------------------

/// In-memory credential store for offline tests.
///
/// Pre-populate via [`InMemoryCredentialStore::insert`].  All test code that
/// exercises the broker MUST use this.  Any test that touches the real OS
/// keyring MUST be `#[ignore]`d.
pub struct InMemoryCredentialStore {
    entries: std::sync::Mutex<HashMap<(String, String), String>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            entries: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Pre-populate an entry for testing.
    pub fn insert(&self, service: &str, account: &str, value: &str) {
        self.entries.lock().unwrap().insert(
            (service.to_string(), account.to_string()),
            value.to_string(),
        );
    }
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        let key = (service.to_string(), account.to_string());
        Ok(self.entries.lock().unwrap().get(&key).cloned())
    }
}

// ---------------------------------------------------------------------------
// OsKeyringStore — production implementation using the `keyring` crate
// ---------------------------------------------------------------------------

/// OS keyring credential store.  Uses the `keyring` crate which delegates to:
/// - **macOS**: Keychain
/// - **Linux**: libsecret / kernel keyring
/// - **Windows**: Windows Credential Store
///
/// This type is only usable in production code, not in tests.  Tests MUST
/// use [`InMemoryCredentialStore`].
pub struct OsKeyringStore;

impl CredentialStore for OsKeyringStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        use keyring::Entry;
        let entry = Entry::new(service, account)
            .map_err(|e| CredentialStoreError::Backend(e.to_string()))?;
        match entry.get_password() {
            Ok(val) => Ok(Some(val)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CredentialStoreError::Backend(e.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// CredentialBroker — the main API surface
// ---------------------------------------------------------------------------

/// Resolves credential requests and injects them into child-process environments.
///
/// ## Usage
///
/// ```rust,ignore
/// let store = Arc::new(InMemoryCredentialStore::new());
/// store.insert("bwoc/github", "token", "ghp_test_value");
/// let broker = CredentialBroker::new(store);
///
/// let requests = vec![CredentialRequest::github_token()];
/// let injected = broker.resolve(&requests)?;
///
/// // Pass to tokio::process::Command
/// command.envs(&injected.env_vars);
/// ```
pub struct CredentialBroker {
    store: Box<dyn CredentialStore>,
}

impl CredentialBroker {
    /// Create a broker with the given credential store.
    pub fn new(store: impl CredentialStore + 'static) -> Self {
        Self {
            store: Box::new(store),
        }
    }

    /// Resolve a list of credential requests.
    ///
    /// Returns a [`ResolvedCredentials`] map that is safe to pass to
    /// `Command::envs`.  Required credentials that cannot be found cause
    /// [`HarnessError::Other`].  Optional missing credentials are silently
    /// skipped.
    ///
    /// **Secrets are never logged here.**
    pub fn resolve(
        &self,
        requests: &[CredentialRequest],
    ) -> Result<ResolvedCredentials, HarnessError> {
        let mut env_vars: HashMap<String, String> = HashMap::new();

        for req in requests {
            match self.store.get(req.keyring_service, req.keyring_account) {
                Ok(Some(value)) => {
                    env_vars.insert(req.env_var.to_string(), value);
                }
                Ok(None) if req.optional => {
                    // Missing optional credential — silently skip.
                }
                Ok(None) => {
                    return Err(HarnessError::Other(format!(
                        "required credential `{}` not found in keyring \
                         (service=`{}`, account=`{}`)",
                        req.env_var, req.keyring_service, req.keyring_account
                    )));
                }
                Err(e) => {
                    return Err(HarnessError::Other(format!(
                        "keyring error for `{}`: {e}",
                        req.env_var
                    )));
                }
            }
        }

        Ok(ResolvedCredentials { env_vars })
    }
}

// ---------------------------------------------------------------------------
// ResolvedCredentials — the injection surface
// ---------------------------------------------------------------------------

/// The result of [`CredentialBroker::resolve`].
///
/// Pass [`ResolvedCredentials::env_vars`] directly to
/// `tokio::process::Command::envs`.
///
/// Use [`ResolvedCredentials::scrubbed_env`] when you need to log or record
/// the child environment — all credential values are replaced with
/// `"[REDACTED]"`.
#[derive(Debug)]
pub struct ResolvedCredentials {
    /// The scoped env vars to inject (env_var_name → value).
    ///
    /// Values are secret — do NOT log, record in telemetry, or include in
    /// any prompt.
    pub env_vars: HashMap<String, String>,
}

impl ResolvedCredentials {
    /// Return a copy of the env-var map with all values replaced by
    /// `"[REDACTED]"`.
    ///
    /// Safe to log or include in telemetry metrics.
    pub fn scrubbed_env(&self) -> HashMap<String, String> {
        self.env_vars
            .keys()
            .map(|k| (k.clone(), "[REDACTED]".to_string()))
            .collect()
    }

    /// Whether any credentials were resolved.
    pub fn is_empty(&self) -> bool {
        self.env_vars.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── InMemoryCredentialStore ───────────────────────────────────────────────

    #[test]
    fn in_memory_store_returns_inserted_value() {
        let store = InMemoryCredentialStore::new();
        store.insert("bwoc/github", "token", "ghp_test_token");
        let val = store.get("bwoc/github", "token").unwrap();
        assert_eq!(val.as_deref(), Some("ghp_test_token"));
    }

    #[test]
    fn in_memory_store_returns_none_for_missing() {
        let store = InMemoryCredentialStore::new();
        let val = store.get("bwoc/github", "token").unwrap();
        assert!(val.is_none());
    }

    // ── CredentialBroker — required credential present ────────────────────────

    #[test]
    fn broker_injects_github_token_when_present() {
        let store = InMemoryCredentialStore::new();
        store.insert("bwoc/github", "token", "ghp_secret_value");
        let broker = CredentialBroker::new(store);

        let resolved = broker
            .resolve(&[CredentialRequest::github_token()])
            .unwrap();

        assert!(resolved.env_vars.contains_key("GITHUB_TOKEN"));
        assert_eq!(
            resolved.env_vars.get("GITHUB_TOKEN").unwrap(),
            "ghp_secret_value"
        );
    }

    // ── CredentialBroker — required credential missing → error ───────────────

    #[test]
    fn broker_errors_on_missing_required_credential() {
        let store = InMemoryCredentialStore::new();
        let broker = CredentialBroker::new(store);

        let err = broker
            .resolve(&[CredentialRequest::github_token()])
            .unwrap_err();

        assert!(
            matches!(err, HarnessError::Other(ref msg) if msg.contains("GITHUB_TOKEN")),
            "expected error mentioning GITHUB_TOKEN, got: {err:?}"
        );
    }

    // ── CredentialBroker — optional credential missing → silently skipped ────

    #[test]
    fn broker_skips_missing_optional_credential() {
        let store = InMemoryCredentialStore::new();
        let broker = CredentialBroker::new(store);

        // NPM_TOKEN is optional.
        let resolved = broker.resolve(&[CredentialRequest::npm_token()]).unwrap();

        // No error; NPM_TOKEN simply absent from the map.
        assert!(!resolved.env_vars.contains_key("NPM_TOKEN"));
    }

    // ── Secret values are NOT in scrubbed_env ────────────────────────────────

    #[test]
    fn scrubbed_env_redacts_all_values() {
        let store = InMemoryCredentialStore::new();
        store.insert("bwoc/github", "token", "ghp_supersecret");
        store.insert("bwoc/npm", "token", "npm_supersecret");
        let broker = CredentialBroker::new(store);

        let requests = vec![
            CredentialRequest::github_token(),
            CredentialRequest::custom("NPM_TOKEN", "bwoc/npm", "token", true),
        ];
        let resolved = broker.resolve(&requests).unwrap();
        let scrubbed = resolved.scrubbed_env();

        // Keys present, values are "[REDACTED]".
        assert_eq!(scrubbed.get("GITHUB_TOKEN").unwrap(), "[REDACTED]");
        assert_eq!(scrubbed.get("NPM_TOKEN").unwrap(), "[REDACTED]");

        // Original map still has the real values (not mutated by scrubbed_env).
        assert_eq!(
            resolved.env_vars.get("GITHUB_TOKEN").unwrap(),
            "ghp_supersecret"
        );
    }

    // ── Secret values must not appear in telemetry ────────────────────────────

    #[test]
    fn telemetry_cannot_observe_credential_values() {
        // Document the invariant:
        // The broker's ResolvedCredentials.env_vars are NEVER passed to any
        // telemetry::TurnMetrics field.  TurnMetrics only holds numeric fields
        // (verified in telemetry tests).  This test confirms that if a caller
        // uses scrubbed_env() for any string-based recording, the secret is
        // absent from that string.

        let store = InMemoryCredentialStore::new();
        store.insert("bwoc/github", "token", "super_secret_token");
        let broker = CredentialBroker::new(store);

        let resolved = broker
            .resolve(&[CredentialRequest::github_token()])
            .unwrap();
        let scrubbed = resolved.scrubbed_env();

        // Convert to a string (as a logger would) — secret must not appear.
        let logged = format!("{scrubbed:?}");
        assert!(
            !logged.contains("super_secret_token"),
            "secret value leaked into logged representation: {logged}"
        );
        assert!(
            logged.contains("[REDACTED]"),
            "expected [REDACTED] in logged representation: {logged}"
        );
    }

    // ── Compose with sandbox env-scrub ────────────────────────────────────────

    #[test]
    fn injected_vars_override_scrubbed_env() {
        // Simulate the compose flow:
        //   scrub_env() → inject credentials → final child env
        //
        // After scrub_env, GITHUB_TOKEN is absent (was not in ENV_ALLOWLIST).
        // The broker then adds it scoped-only.
        use crate::sandbox::scrub_env;

        let store = InMemoryCredentialStore::new();
        store.insert("bwoc/github", "token", "ghp_injected");
        let broker = CredentialBroker::new(store);

        let mut child_env = scrub_env();
        // GITHUB_TOKEN must not be in the scrubbed env.
        assert!(!child_env.contains_key("GITHUB_TOKEN"));

        // Now inject the broker's resolved creds.
        let resolved = broker
            .resolve(&[CredentialRequest::github_token()])
            .unwrap();
        child_env.extend(resolved.env_vars.clone());

        // The final env has GITHUB_TOKEN.
        assert_eq!(child_env.get("GITHUB_TOKEN").unwrap(), "ghp_injected");
    }

    // ── OS keyring test is ignored (requires real keyring at test time) ───────

    /// This test is intentionally `#[ignore]`d.  Run with
    /// `cargo test -- --ignored` on a machine with a configured OS keyring.
    #[test]
    #[ignore]
    fn os_keyring_store_roundtrip() {
        use keyring::Entry;

        let service = "bwoc-test-service";
        let account = "bwoc-test-account";
        let secret = "bwoc-test-secret-value";

        // Write a test entry.
        let entry = Entry::new(service, account).unwrap();
        entry.set_password(secret).unwrap();

        // Read it back via the store.
        let store = OsKeyringStore;
        let val = store.get(service, account).unwrap();
        assert_eq!(val.as_deref(), Some(secret));

        // Clean up.
        entry.delete_credential().unwrap();
    }
}
