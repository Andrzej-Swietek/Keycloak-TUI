use super::*;

#[test]
fn false_flags_are_omitted_but_true_ones_are_sent() {
    let pairs = Query::new().with_exact(false).to_pairs();
    assert!(!pairs.iter().any(|(k, _)| k == "exact"));

    let pairs = Query::new().with_exact(true).with_brief(true).to_pairs();
    assert!(pairs.contains(&("exact".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("briefRepresentation".to_owned(), "true".to_owned())));
}

#[test]
fn attribute_queries_use_keycloaks_space_separated_q_syntax() {
    let pairs = Query::new()
        .with_attribute("department", "platform")
        .with_attribute("tier", "gold")
        .to_pairs();

    let q = pairs.iter().find(|(k, _)| k == "q").unwrap();
    assert_eq!(q.1, "department:platform tier:gold");
}

#[test]
fn paging_keeps_filters_and_never_underflows() {
    let query = Query::page(0, 50).with_search("alice");
    let next = query.clone().next_page();
    assert_eq!(next.first, 50);
    assert_eq!(next.search.as_deref(), Some("alice"));
    assert_eq!(next.page_index(), 1);

    assert_eq!(query.previous_page().first, 0);
}

#[test]
fn blank_searches_are_dropped_rather_than_sent_as_empty() {
    assert!(Query::new().with_search("   ").search.is_none());
}
