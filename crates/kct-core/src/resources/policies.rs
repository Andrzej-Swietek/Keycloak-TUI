use super::prelude::*;

pub(crate) const EVENT_CONFIG: Resource = Resource {
    id: "event-config",
    singular: "Event configuration",
    plural: "Event configuration",
    section: Section::Realm,
    collection: ep::EVENTS_CONFIG,
    item: Some(ep::EVENTS_CONFIG),
    shape: Shape::Singleton,
    fixed_params: &[],
    key: Key::custom("__singleton"),
    new_object: None,
    label: &[],
    columns: &[],
    search: Search::None,
    ops: Ops::RW,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Configuration",
            fields: &[
                Field::new("User events enabled", "/eventsEnabled", FieldKind::Bool),
                Field::new("User events expiration", "/eventsExpiration", FieldKind::Seconds),
                Field::new("Enabled event types", "/enabledEventTypes", FieldKind::List),
                Field::new("Admin events enabled", "/adminEventsEnabled", FieldKind::Bool),
                Field::new("Admin event details", "/adminEventsDetailsEnabled", FieldKind::Bool)
                    .help("Stores the full representation of each admin change — large, but the only way to see what actually changed."),
                Field::new("Event listeners", "/eventsListeners", FieldKind::List)
                    .help("Provider ids of installed EventListenerProvider SPIs."),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new("edit", "Edit event configuration", Verb::Edit, Danger::Confirm)],
    keywords: &["audit", "logging", "event listener", "kafka", "jboss-logging"],
    summary: "Which events this realm records, and who listens to them.",
};
