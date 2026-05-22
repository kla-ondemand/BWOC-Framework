//! Localized output via [Project Fluent](https://projectfluent.org/).
//!
//! Locale files (`crates/bwoc-cli/locales/<lang>/cli.ftl`) are embedded into
//! the binary at compile time via `include_str!`, so a distributed `bwoc`
//! does not need to find them on disk. Adding a new language is two edits:
//! drop `<lang>/cli.ftl` next to the existing files, then add a match arm
//! in `bundle_for`.
//!
//! This iter wires the infrastructure plus ONE message as proof. Converting
//! the remaining `println!` literals across the CLI is a follow-up.

use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

const EN_CLI_FTL: &str = include_str!("../locales/en/cli.ftl");
const TH_CLI_FTL: &str = include_str!("../locales/th/cli.ftl");

/// Build a Fluent bundle for `lang`. Unsupported codes fall back to `en`.
pub fn bundle_for(lang: &str) -> FluentBundle<FluentResource> {
    let (ftl, resolved) = match lang {
        "th" => (TH_CLI_FTL, "th"),
        _ => (EN_CLI_FTL, "en"),
    };

    let resource =
        FluentResource::try_new(ftl.to_string()).expect("embedded .ftl is valid at compile time");
    let langid: LanguageIdentifier = resolved.parse().expect("static langid string parses");
    let mut bundle = FluentBundle::new(vec![langid]);
    // Default Fluent wraps interpolated args with Unicode bidirectional
    // isolation marks; disable that so terminal output is plain ASCII/UTF-8.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .expect("embedded .ftl is consistent at compile time");
    bundle
}

/// Look up a message by key with no arguments. Returns the localized string
/// or a visible placeholder `«missing key: <key>»` if the key isn't defined
/// — surfaces gaps during development without panicking.
pub fn t(bundle: &FluentBundle<FluentResource>, key: &str) -> String {
    t_inner(bundle, key, None)
}

/// Look up a message by key with named string arguments. Args are passed
/// as `&[(name, value)]` slices for ergonomics at the call site:
///
/// ```ignore
/// t_with(&bundle, "init.success-title", &[("path", "/tmp/ws")])
/// ```
///
/// All arg values are treated as strings; if you need numbers or other
/// Fluent value types later, expose a richer API at that point.
pub fn t_with(bundle: &FluentBundle<FluentResource>, key: &str, args: &[(&str, &str)]) -> String {
    let mut fargs = FluentArgs::new();
    for (name, value) in args {
        fargs.set(*name, FluentValue::from(*value));
    }
    t_inner(bundle, key, Some(&fargs))
}

fn t_inner(bundle: &FluentBundle<FluentResource>, key: &str, args: Option<&FluentArgs>) -> String {
    let Some(msg) = bundle.get_message(key) else {
        return format!("«missing key: {key}»");
    };
    let Some(pattern) = msg.value() else {
        return format!("«no value for key: {key}»");
    };
    let mut errors = vec![];
    let cow: Cow<str> = bundle.format_pattern(pattern, args, &mut errors);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_bundle_returns_default_help_hint() {
        let b = bundle_for("en");
        let s = t(&b, "default-help-hint");
        assert!(s.contains("bwoc"), "got: {s:?}");
        assert!(s.contains("--help"), "got: {s:?}");
        assert!(s.contains("try"), "EN should contain 'try', got: {s:?}");
    }

    #[test]
    fn th_bundle_returns_localized_help_hint() {
        let b = bundle_for("th");
        let s = t(&b, "default-help-hint");
        assert!(s.contains("bwoc"), "got: {s:?}");
        assert!(
            s.contains("ลองใช้"),
            "TH should contain Thai 'ลองใช้', got: {s:?}"
        );
        assert!(
            !s.contains("try"),
            "TH should NOT contain English 'try', got: {s:?}"
        );
    }

    #[test]
    fn unknown_lang_falls_back_to_en() {
        let b = bundle_for("zz");
        let s = t(&b, "default-help-hint");
        assert!(
            s.contains("try"),
            "unknown lang should fall back to EN, got: {s:?}"
        );
    }

    #[test]
    fn missing_key_returns_marker() {
        let b = bundle_for("en");
        let s = t(&b, "no-such-key-at-all");
        assert!(s.starts_with("«missing"), "got: {s:?}");
    }

    #[test]
    fn t_with_interpolates_named_args() {
        // `init-success-title` is a real key with a { $path } arg.
        let b = bundle_for("en");
        let s = t_with(&b, "init-success-title", &[("path", "/tmp/wks")]);
        assert!(
            s.contains("/tmp/wks"),
            "expected path to appear, got: {s:?}"
        );
    }
}
