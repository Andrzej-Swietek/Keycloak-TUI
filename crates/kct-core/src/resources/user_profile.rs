//! The declarative user profile, taken apart.
//!
//! Keycloak keeps the whole profile in one document: the attribute definitions, the display
//! groups they sort into, and the policy for attributes nobody declared. Rendering that as a
//! single JSON field makes the one thing operators actually do — add an attribute, mark it
//! required, restrict who may edit it — a text-editing exercise. So the two arrays inside it
//! are presented as collections, and a write puts the whole document back with just that one
//! element changed.

use super::prelude::*;
use crate::path::PathParams;

pub(crate) const USER_PROFILE_ATTRIBUTES: Resource = Resource {
    id: "user-profile-attributes",
    singular: "Attribute",
    plural: "Profile attributes",
    section: Section::Realm,
    collection: ep::USER_PROFILE,
    item: None,
    shape: Shape::ArrayAt("/attributes"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: Some(new_attribute),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Display name", Projection::At("/displayName"), Width::Flex(3)),
        Column::new("Group", Projection::At("/group"), Width::Flex(2)).toned(Tone::Muted),
        Column::new(
            "Required",
            Projection::Join {
                pointer: "/required/roles",
                separator: ",",
            },
            Width::Fixed(12),
        )
        .toned(Tone::Flag),
        Column::new(
            "Multivalued",
            Projection::Bool {
                pointer: "/multivalued",
                yes: "yes",
                no: "no",
            },
            Width::Fixed(11),
        ),
        Column::new(
            "Can edit",
            Projection::Join {
                pointer: "/permissions/edit",
                separator: ",",
            },
            Width::Flex(2),
        )
        .toned(Tone::Muted),
        Column::new("Validators", Projection::Count("/validations"), Width::Fixed(10))
            .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Definition",
            fields: &[
                Field::new("Name", "/name", FieldKind::Text)
                    .help("The attribute key as stored on the user. Renaming one orphans values already written under the old key."),
                Field::new("Display name", "/displayName", FieldKind::Text)
                    .help("Shown on forms. `${key}` resolves against the theme's message bundle."),
                Field::new("Group", "/group", FieldKind::Text)
                    .help("Name of an attribute group to sort this under; see the Groups screen."),
                Field::new("Multivalued", "/multivalued", FieldKind::Bool),
                Field::new("Required for roles", "/required/roles", FieldKind::List)
                    .help("`user`, `admin`, or both. Empty means the attribute is optional."),
                Field::new("Required for scopes", "/required/scopes", FieldKind::List)
                    .help("Required only when one of these client scopes is requested."),
                Field::new("Can view", "/permissions/view", FieldKind::List)
                    .help("`user`, `admin`. An attribute nobody may view is invisible everywhere."),
                Field::new("Can edit", "/permissions/edit", FieldKind::List)
                    .help("`user`, `admin`. Without `admin` even this tool cannot write it."),
                Field::new("Shown when scopes", "/selector/scopes", FieldKind::List)
                    .help("Render this attribute only for these client scopes."),
                Field::new("Validators", "/validations", FieldKind::Json)
                    .help("`{\"length\": {\"min\": 3, \"max\": 255}}` — validator id to its configuration."),
                Field::new("Annotations", "/annotations", FieldKind::StringMap)
                    .help("Hints for the login theme: input type, placeholder, help text."),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "delete",
        "Delete attribute",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &[
        "declarative user profile",
        "attribute validation",
        "required attribute",
        "unmanaged attributes",
        "custom user attribute",
    ],
    summary: "Attributes the realm declares on users, with their validators and permissions.",
};

pub(crate) const USER_PROFILE_GROUPS: Resource = Resource {
    id: "user-profile-groups",
    singular: "Attribute group",
    plural: "Attribute groups",
    section: Section::Realm,
    collection: ep::USER_PROFILE,
    item: None,
    shape: Shape::ArrayAt("/groups"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: Some(new_group),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(2)).toned(Tone::Primary),
        Column::new("Header", Projection::At("/displayHeader"), Width::Flex(3)),
        Column::new(
            "Description",
            Projection::At("/displayDescription"),
            Width::Flex(4),
        )
        .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Definition",
            fields: &[
                Field::new("Name", "/name", FieldKind::Text)
                    .help("Referenced by an attribute's Group field."),
                Field::new("Header", "/displayHeader", FieldKind::Text),
                Field::new("Description", "/displayDescription", FieldKind::Text),
                Field::new("Annotations", "/annotations", FieldKind::StringMap),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "delete",
        "Delete attribute group",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &["attribute group", "profile section", "form layout"],
    summary: "Headings the login and account forms sort declared attributes under.",
};

/// A new attribute readable by both audiences and writable only by an admin.
///
/// Keycloak rejects an attribute with no permissions, and one that a user may edit is a
/// larger decision than creating a row: the safe default is the narrow one.
fn new_attribute(name: &str, _params: &PathParams) -> Value {
    serde_json::json!({
        "name": name,
        "displayName": name,
        "multivalued": false,
        "permissions": {"view": ["admin", "user"], "edit": ["admin"]},
        "validations": {},
        "annotations": {},
    })
}

fn new_group(name: &str, _params: &PathParams) -> Value {
    serde_json::json!({
        "name": name,
        "displayHeader": name,
        "displayDescription": "",
        "annotations": {},
    })
}
