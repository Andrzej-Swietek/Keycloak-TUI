# keycloak-tui (`kct`)

A terminal UI for administering Keycloak — built for the case where the admin console is not
an option: a jump host, an SSH session into a locked-down environment, an incident at 3am.

Verified against **Keycloak 26.8.1** (and 26.7.3, 26.6.1 in CI); floor is 25.

```
kct -s https://sso.example.com -u admin           # prompts for the password
kct -c prod --screen user-federation              # a saved context, opening on LDAP
kct -s http://localhost:8080 --client-secret ...  # a service account
```

## What it does

Every Keycloak collection is described once as **data** and rendered by one engine, so the
tool covers the admin surface rather than a curated slice of it:

| Area | Screens |
|---|---|
| Manage | users (credentials, groups, sessions, consents, identity links, realm and client role mappings), groups (children, members), organizations (members and their membership type, identity providers and how they enrol users, domain routing) |
| Configure | clients (roles, mappers, scopes, effective mappers, sessions, authz resources/scopes/policies/permissions), client scopes, realm roles, identity providers, user federation, authentication flows, required actions, initial access tokens |
| Realm | realm settings (general, login, sessions, tokens, security, flows & OTP, email, themes), keys, key providers, user profile, client policies and profiles, event configuration |
| Observe | sessions per client, login events, admin events |
| Server | realms, feature flags, SPIs and providers, server info |

A pixel-art mascot in the bottom-right corner doubles as a status light: it works while a request is in
flight, smiles after a write lands, frowns on a failure, and looks alarmed while a destructive
confirmation is open. It is drawn with half-block characters — each cell carries an upper and a lower pixel as its
foreground and background, so the text grid becomes a 16×16 sprite — and uses its own
256-colour palette rather than the terminal's sixteen. Four characters ship:
`--mascot twintails|cat|robot|ghost|none`, or cycle them from the palette. It hides itself on
a terminal too small to spare the room.

Adding one is a single file: the body is sixteen strings of palette keys, and the face
geometry is shared, so the expressions are written once and work on every character.

Keys: `:` command line (vim `:w` `:q` `:q!` `:wq`, or fuzzy palette search) · `a` this
screen's actions · `/` filter · `s` server-side search · `⏎` open · `n` new · `x` delete ·
`w` save · `L` request log · `?` help.

## `kct doctor` — resilience you can measure

```
kct -s https://sso.example.com -u admin -r acme doctor
```

Probes every screen against a **live server**, compares the catalog to what came back, and
exits non-zero when something is genuinely broken. Run it in CI against the Keycloak version
you deploy and an upgrade that moves an endpoint fails the build instead of a support ticket.

```
0 broken, 205 coverage gaps, 13 notes — catalog matches this server

Fields reachable only through the JSON tab:
    95  realms
    65  realm-settings
    14  server-info
```

It distinguishes what it can prove from what it cannot. A failed listing or a gate on a
feature the server does not define is **broken**. A field only the JSON tab reaches is a
**coverage gap** — nothing is lost, but a form could be better. An optional field that
happened to be unset on every sampled object is a **note**, because absence is not evidence.

Its first run against 26.7.3 found a real bug: Keycloak now reports feature names as
`ORGANIZATION` where the `--features=` flag spells them `organization`, so every feature gate
had quietly become "this feature does not exist". Lookups are normalised across both
spellings now, with a test pinning each. It also found five LDAP settings the server
advertises that no form offered; they are offered now.

## Why it should survive a Keycloak upgrade

- **Nothing is narrowed on the way through.** A representation is edited as the server's own
  JSON and written back merged, so a field this build has never heard of survives a save.
  Tools that deserialise into structs quietly drop those fields on update.
- **The JSON tab is an editor, not a viewer.** It flattens an object into
  `pointer = value` lines, every one of them selectable and editable — including fields added
  by a newer Keycloak that no form here knows about.
- **Capabilities come from the server, not from the build.** Screens are gated on a live
  `serverinfo` read; an *unknown* answer always permits the attempt, so failing detection
  never costs an operator a screen. On 26.7 a realm-scoped admin gets `serverinfo` without
  `systemInfo` — features still work, the version shows as `version hidden`.
- **Endpoints live in one module.** A relocated path is a one-line diff, not a hunt.
- **The catalog is checked by tests**: ids unique, every nested screen reachable, every screen
  able to render a row, every object keeping its JSON escape hatch. `kct doctor` then checks it
  against a real server, and a live test creates one of every creatable resource.

## Editing

- **Fields that accept a fixed vocabulary suggest it.** Choices come from the catalog, themes
  and provider ids from the live server — so a theme that is not installed cannot be typed in
  by mistake. Tab walks the suggestions.
- **Attribute maps get a key–value editor**, not a JSON blob. It knows the two shapes Keycloak
  uses — `Map<String, List<String>>` for users, groups, roles and organizations, `Map<String,
  String>` for realms and clients — because writing the wrong one is accepted and then ignored.
- **New objects use a per-resource template.** A generic "name plus enabled" body is rejected
  by most of Keycloak's representations: a component has no `enabled` field, a client scope
  needs a protocol, an LDAP mapper needs its parent. A live test creates one of every
  creatable resource to keep the templates honest.
- **Granting works from where you are.** Assigning a realm or client role, adding someone to
  a group or attaching a client scope is an action on the user, group or client you are
  looking at — no descending two levels first. Multi-select, one request.
- **Only valid pairings are offered.** The picker subtracts what is already attached and what
  the server would refuse: a client scope already attached as *optional* cannot also be
  *default*, and a SAML scope on an OIDC client is accepted with a 204 and then ignored.
- **A new LDAP provider starts disabled.** An enabled one with a blank URL is consulted by
  every user lookup in the realm and turns ordinary operations into opaque 500s.

## Things it tells you that the console does not

- A 403 names the `realm-management` role you are missing, per endpoint.
- Saving an attribute the realm's user profile would **silently discard** warns first —
  Keycloak returns 201 and drops it.
- A protocol mapper's row shows which token surfaces it actually emits on
  (`access,id,userinfo,introspect`), which is usually the answer to "why is my claim missing".
- Preview and deprecated features that are *enabled* are reported at startup.
- `--dry-run` refuses every write and says what would have been sent.
- A 404 that really means "the realm has not opted in" says so, and names the switch.
- An opaque 500 points at user federation, which is what causes it far more often than not.
- *Probe SCIM as this client* fetches the client's token, calls `/scim/v2` with it and names
  the missing piece: the realm switch (404), the `…/scim/v2` audience (401), or the
  `view-users`/`manage-users` role on the service account (403). *Prepare client for SCIM*
  adds the audience mapper.
- Saving an identity provider with its masked secret after changing the token URL, client ID
  or auth method is refused by 26.8; the hint says to type the secret again rather than
  "the payload was rejected".
- The token inspector says when Keycloak issued a *lightweight* access token, so the missing
  `sub`, `aud` and roles read as a design choice rather than a bug.

## Design

```
kct-core     domain: catalog, capability gating, request planning, ports. No I/O.
kct-client   the only adapter: reqwest. Implements one trait.
kct-tui      Elm architecture: update is pure, effects are data, view renders &Model.
kct          CLI, contexts, wiring.
```

One trait (`Gateway`) is all a transport must implement; `Rest` and `AdminApi` are
blanket-implemented on top, so a test double is a single method yet exercises everything.
`update` is `(Model, Msg) -> (Model, Vec<Effect>)` — no `&mut`, no I/O, fully testable.

## Configuration

`kct config-path` prints the location. Contexts work like a kubeconfig; passwords never go in
the file.

```toml
default = "dev"

[contexts.dev]
server = "http://localhost:8080"
realm  = "master"

[contexts.prod]
server     = "https://sso.example.com"
realm      = "acme"
auth_realm = "master"
username   = "sre-admin"
screen     = "user-federation"
```

## Development

```
cargo test                                                  # 226 tests, no server needed
cargo clippy --all-targets                                  # clean under clippy::pedantic

docker run -d -p 18080:8080 -e KC_BOOTSTRAP_ADMIN_USERNAME=admin \
  -e KC_BOOTSTRAP_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:26.8.1 start-dev
KCT_TEST_SERVER=http://localhost:18080 cargo test -p kct-client --tests
cargo run -- -s http://localhost:18080 -u admin -p admin doctor   # drift check
```

Adding a screen means adding a `Resource` value to `kct-core/src/resources/`. No new
rendering code.

## Token and file tools

`:` then *inspect* decodes the token this session is presenting — scope, audience, roles,
expiry, and whether it is a lightweight token. *Get a token as this client* on a confidential
client fetches its secret, exchanges it for a token and decodes that, which answers "what does
this client actually receive" in one step. *Copy last call as curl* produces a command that
references `$KC_TOKEN` rather than pasting a bearer into your clipboard history.

*Export* opens the realm representation in a viewer; `s` writes it to a file. *Import* reads
one back as a partial import, skipping objects that already exist rather than overwriting them.
