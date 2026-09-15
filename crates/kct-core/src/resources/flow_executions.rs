use super::prelude::*;

pub(crate) const FLOW_EXECUTIONS: Resource = Resource {
    id: "flow-executions",
    singular: "Execution",
    plural: "Executions",
    section: Section::Configure,
    collection: ep::AUTH_FLOW_EXECUTIONS,
    item: Some(ep::AUTH_EXECUTION),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/displayName", "/providerId"],
    columns: &[
        Column::new("Step", Projection::Compute(execution_label), Width::Flex(5))
            .toned(Tone::Primary),
        Column::new(
            "Requirement",
            Projection::At("/requirement"),
            Width::Fixed(14),
        ),
        Column::new("Provider", Projection::At("/providerId"), Width::Flex(3)).toned(Tone::Muted),
        Column::new(
            "Configurable",
            Projection::Bool {
                pointer: "/configurable",
                yes: "yes",
                no: "",
            },
            Width::Fixed(13),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "add-step",
            "Add a step",
            Verb::Special("add-execution"),
            Danger::Confirm,
        )
        .keywords(&["authenticator", "execution", "add step", "insert"]),
        Action::new(
            "add-subflow",
            "Add a sub-flow",
            Verb::Special("add-sub-flow"),
            Danger::Confirm,
        )
        .keywords(&["nested flow", "conditional", "group of steps"]),
        Action::new(
            "set-requirement",
            "Change requirement",
            Verb::Special("set-execution-requirement"),
            Danger::Confirm,
        )
        .keywords(&["required", "alternative", "disabled", "conditional"]),
        Action::new(
            "delete",
            "Remove execution",
            Verb::Delete,
            Danger::Destructive,
        ),
    ],
    keywords: &["authenticator", "step", "mfa step", "conditional"],
    summary: "Ordered steps of this authentication flow.",
};

/// Indent one execution by its depth, rendering the flattened tree as a tree.
pub(crate) fn execution_label(row: &Value) -> String {
    let level = usize::try_from(row.get("level").and_then(Value::as_u64).unwrap_or(0)).unwrap_or(0);
    let name = row
        .get("displayName")
        .and_then(Value::as_str)
        .or_else(|| row.get("providerId").and_then(Value::as_str))
        .unwrap_or("(step)");

    format!("{}{name}", "  ".repeat(level))
}
