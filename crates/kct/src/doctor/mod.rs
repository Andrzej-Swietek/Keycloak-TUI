//! `kct doctor` — check the catalog against a live server.

mod probe;

use anyhow::Result;
use kct_client::KeycloakClient;
use kct_core::catalog::Ops;
use kct_core::doctor::{self, Finding, Report, Severity};
use kct_core::ports::{AdminApi, Context};
use std::collections::BTreeMap;

use kct_core::{Query, resources};

use probe::Parents;

/// Probe every screen, compare what came back against the catalog, and report the drift.
pub async fn run(client: &KeycloakClient, realm: &str) -> Result<Report> {
    let context = Context::realm(realm);
    let profile = client.server_profile(realm).await.unwrap_or_default();
    let parents = Parents::sample(client, &context).await;

    let mut report = Report::default().with(doctor::check_feature_gates(&profile));

    for resource in resources::CATALOG {
        if !resource.ops.has(Ops::LIST) {
            continue;
        }
        let support = profile.evaluate(&resource.requirement);
        if !support.is_permitted() {
            report = report.with([Finding::note(
                resource.id,
                format!("skipped: {}", support.reason().unwrap_or("unavailable")),
            )]);
            continue;
        }

        let Some(probe_context) = parents
            .context_for(resource, &context)
            .or_else(|| parents.owner_context(&context).filter(|_| owns(resource)))
        else {
            report = report.with([Finding::note(
                resource.id,
                "skipped: no object in this realm to probe it against",
            )]);
            continue;
        };
        let probe_context = parents.with_owner_kind(probe_context, resource);

        match client
            .list(resource, &probe_context, &Query::page(0, 3))
            .await
        {
            Ok(page) => {
                report = report.with(doctor::compare_fields(resource, &page.items));
                report = report.with(doctor::compare_kinds(resource, &page.items));
            }
            // A realm that has not opted in answers 404; that is configuration, not drift.
            Err(error) if error.to_string().contains("not enabled") => {
                report = report.with([Finding::note(
                    resource.id,
                    "skipped: the realm has not enabled this feature",
                )]);
            }
            Err(error) => {
                report = report.with([Finding::broken(
                    resource.id,
                    format!("listing failed: {error}"),
                )]);
            }
        }
    }

    // Component-backed screens have a real schema in serverinfo; use it.
    for (id, spi, provider) in [
        (
            "user-federation",
            "org.keycloak.storage.UserStorageProvider",
            "ldap",
        ),
        (
            "key-providers",
            "org.keycloak.keys.KeyProvider",
            "rsa-generated",
        ),
    ] {
        if let Some(resource) = resources::by_id(id) {
            report = report.with(doctor::compare_component(resource, &profile, spi, provider));
        }
    }

    Ok(report.sorted())
}

/// Resources whose path is addressed by an owner: a user or a group, not a parent screen.
fn owns(resource: &kct_core::catalog::Resource) -> bool {
    resource.collection_slots().contains(&"owner")
}

/// Print the report the way a CI log wants it.
///
/// Breakages are listed individually because each one needs a decision. Coverage gaps are
/// summarised per screen: two hundred lines saying "this field is only in the JSON tab" is
/// noise, "realm-settings renders 52 of 141 fields" is a number worth watching.
pub fn print(report: &Report, verbose: bool) {
    for finding in &report.findings {
        match finding.severity {
            Severity::Broken => println!("{finding}"),
            Severity::Missing | Severity::Note if verbose => println!("{finding}"),
            _ => {}
        }
    }

    if !verbose {
        print_coverage(report);
    }

    println!(
        "\n{} broken, {} coverage gaps, {} notes — {}",
        report.count(Severity::Broken),
        report.count(Severity::Missing),
        report.count(Severity::Note),
        if report.is_healthy() {
            "catalog matches this server"
        } else {
            "the catalog is out of step with this server"
        }
    );
    if !verbose && report.count(Severity::Missing) > 0 {
        println!("Run with --verbose to see every gap.");
    }
}

fn print_coverage(report: &Report) {
    let mut per_screen: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in &report.findings {
        if finding.severity == Severity::Missing {
            *per_screen.entry(finding.resource.as_str()).or_default() += 1;
        }
    }
    if per_screen.is_empty() {
        return;
    }

    let mut ranked: Vec<_> = per_screen.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    println!("Fields reachable only through the JSON tab:");
    for (screen, count) in ranked.iter().take(10) {
        println!("  {count:>4}  {screen}");
    }
    if ranked.len() > 10 {
        println!("  … and {} more screens", ranked.len() - 10);
    }
}
