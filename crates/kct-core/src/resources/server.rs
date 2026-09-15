use super::prelude::*;

pub(crate) const SERVER_FEATURES: Resource = Resource {
    id: "server-features",
    singular: "Feature",
    plural: "Feature flags",
    section: Section::Server,
    collection: ep::SERVER_INFO,
    item: None,
    shape: Shape::ArrayAt("/features"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: None,
    label: &["/name"],
    columns: &[
        Column::new("Feature", Projection::At("/name"), Width::Flex(4)).toned(Tone::Primary),
        Column::new(
            "Enabled",
            Projection::Bool {
                pointer: "/enabled",
                yes: "ON",
                no: "off",
            },
            Width::Fixed(8),
        )
        .toned(Tone::Flag),
        Column::new("Support", Projection::At("/type"), Width::Fixed(24)).toned(Tone::Warn),
        Column::new("Label", Projection::At("/label"), Width::Flex(6)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &[
        "preview",
        "experimental",
        "deprecated",
        "kc.sh build",
        "--features",
    ],
    summary: "Feature flags on this server, with their support status.",
};

pub(crate) const SERVER_PROVIDERS: Resource = Resource {
    id: "server-providers",
    singular: "SPI",
    plural: "SPIs & providers",
    section: Section::Server,
    collection: ep::SERVER_INFO,
    item: None,
    shape: Shape::EntriesAt("/providers"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: None,
    label: &["/name"],
    columns: &[
        Column::new("SPI", Projection::At("/name"), Width::Flex(4)).toned(Tone::Primary),
        Column::new(
            "Internal",
            Projection::Bool {
                pointer: "/internal",
                yes: "yes",
                no: "",
            },
            Width::Fixed(9),
        )
        .toned(Tone::Muted),
        Column::new(
            "Providers",
            Projection::Count("/providers"),
            Width::Fixed(10),
        ),
        Column::new(
            "Implementations",
            Projection::Compute(provider_ids),
            Width::Flex(8),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["spi", "extension", "custom provider", "jar", "deployed"],
    summary: "Every SPI and the providers registered for it — including your own extensions.",
};

pub(crate) fn provider_ids(row: &Value) -> String {
    row.get("providers")
        .and_then(Value::as_object)
        .map(|providers| providers.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default()
}

pub(crate) const SERVER_INFO: Resource = Resource {
    id: "server-info",
    singular: "Server info",
    plural: "Server info",
    section: Section::Server,
    collection: ep::SERVER_INFO,
    item: Some(ep::SERVER_INFO),
    shape: Shape::Singleton,
    fixed_params: &[],
    key: Key::custom("__singleton"),
    new_object: None,
    label: &[],
    columns: &[],
    search: Search::None,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "System",
            fields: &[
                Field::new(
                    "Keycloak version",
                    "/systemInfo/version",
                    FieldKind::ReadOnly,
                ),
                Field::new("Server time", "/systemInfo/serverTime", FieldKind::ReadOnly),
                Field::new("Uptime", "/systemInfo/uptime", FieldKind::ReadOnly),
                Field::new(
                    "Java version",
                    "/systemInfo/javaVersion",
                    FieldKind::ReadOnly,
                ),
                Field::new("Java VM", "/systemInfo/javaVm", FieldKind::ReadOnly),
                Field::new("OS", "/systemInfo/osName", FieldKind::ReadOnly),
                Field::new(
                    "OS architecture",
                    "/systemInfo/osArchitecture",
                    FieldKind::ReadOnly,
                ),
                Field::new(
                    "File encoding",
                    "/systemInfo/fileEncoding",
                    FieldKind::ReadOnly,
                ),
                Field::new(
                    "Free memory",
                    "/memoryInfo/freeFormated",
                    FieldKind::ReadOnly,
                ),
                Field::new(
                    "Total memory",
                    "/memoryInfo/totalFormated",
                    FieldKind::ReadOnly,
                ),
                Field::new("Profile", "/profileInfo/name", FieldKind::ReadOnly),
            ],
        },
        Tab::Raw,
    ],
    actions: &[],
    keywords: &[
        "version",
        "uptime",
        "memory",
        "jvm",
        "diagnostics",
        "serverinfo",
    ],
    summary: "Version, runtime and capability report for the server itself.",
};
