use super::prelude::*;

pub(crate) const GROUP_MEMBERS: Resource = Resource {
    id: "group-members",
    singular: "Member",
    plural: "Members",
    section: Section::Manage,
    collection: ep::GROUP_MEMBERS,
    item: Some(ep::USER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/username", "/id"],
    columns: USER_COLUMNS,
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["who is in", "membership"],
    summary: "Users that are direct members of this group.",
};

const ORG_MEMBER_COLUMNS: &[Column] = &[
    Column::new("Username", Projection::At("/username"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Email", Projection::At("/email"), Width::Flex(3)),
    Column::new("Name", Projection::Compute(full_name), Width::Flex(3)),
    Column::new(
        "Membership",
        Projection::At("/membershipType"),
        Width::Fixed(10),
    )
    .toned(Tone::Muted),
];

pub(crate) const ORG_MEMBERS: Resource = Resource {
    id: "org-members",
    singular: "Member",
    plural: "Members",
    section: Section::Manage,
    collection: ep::ORGANIZATION_MEMBERS,
    item: Some(ep::USER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/username", "/email", "/id"],
    columns: ORG_MEMBER_COLUMNS,
    search: Search::Server,
    ops: Ops::RO.with(Ops::CREATE).with(Ops::DELETE),
    requirement: KC_26,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new("add", "Add existing user", Verb::Create, Danger::Confirm),
        Action::new(
            "membership",
            "Set membership type",
            Verb::Special("set-member-type"),
            Danger::Confirm,
        )
        .needs(KC_26_8)
        .keywords(&["managed", "unmanaged"]),
        Action::new(
            "delete",
            "Remove from organization",
            Verb::Delete,
            Danger::Confirm,
        ),
    ],
    keywords: &["tenant members"],
    summary: "Users belonging to this organization.",
};
