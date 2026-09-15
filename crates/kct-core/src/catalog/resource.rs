use serde_json::Value;

use crate::capability::Requirement;
use crate::json;
use crate::path::{PathParams, Template};

use super::action::Action;
use super::column::Column;
use super::new_object::NewObject;
use super::ops::{Key, Ops, Search};
use super::section::Section;
use super::shape::Shape;
use super::tab::Tab;

/// A fixed query parameter. Values may contain `{slot}` placeholders, resolved from the same
/// parameters as the path, which is how nested component collections are expressed.
pub type FixedParam = (&'static str, &'static str);

/// One addressable collection in the admin API, described entirely as data.
#[derive(Debug, Clone, Copy)]
pub struct Resource {
    /// Stable internal identifier, used in routes and config.
    pub id: &'static str,
    pub singular: &'static str,
    pub plural: &'static str,
    pub section: Section,
    pub collection: Template,
    /// Item endpoint. `None` means "collection + `/{key}`".
    pub item: Option<Template>,
    pub shape: Shape,
    pub fixed_params: &'static [FixedParam],
    pub key: Key,
    /// How to build a valid minimal object for a create. `None` falls back to the key field
    /// alone, which is right only for the handful of resources that need nothing else.
    pub new_object: Option<NewObject>,
    /// Pointers tried in order to produce a human label for a row.
    pub label: &'static [&'static str],
    pub columns: &'static [Column],
    pub search: Search,
    pub ops: Ops,
    pub requirement: Requirement,
    pub tabs: &'static [Tab],
    pub actions: &'static [Action],
    /// What an operator might *call* this thing, for the command palette.
    pub keywords: &'static [&'static str],
    pub summary: &'static str,
}

/// Resources are identified by their catalog id, which is unique by construction.
impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Resource {}

impl std::hash::Hash for Resource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Resource {
    /// Whether items are elements of the collection document, with no URL of their own.
    ///
    /// True for a [`Shape::ArrayAt`] resource that declares no `item` template — the
    /// declarative user profile's attributes and groups. Reads use the row the listing
    /// returned; writes go through [`Resource::with_item`].
    #[must_use]
    pub const fn embeds_items(&self) -> bool {
        self.item.is_none() && matches!(self.shape, Shape::ArrayAt(_))
    }

    /// `document` with the item keyed `id` set to `element`, or removed when `element` is
    /// `None`. Returns `None` unless [`Resource::embeds_items`].
    ///
    /// Everything outside the addressed item is carried through unchanged.
    #[must_use]
    pub fn with_item(&self, document: Value, id: &str, element: Option<Value>) -> Option<Value> {
        let Shape::ArrayAt(array) = self.shape else {
            return None;
        };
        self.item
            .is_none()
            .then(|| json::splice(document, array, &format!("/{}", self.key.0), id, element))
    }

    /// A body that creates one of these, named `name`.
    #[must_use]
    pub fn draft_new(&self, name: &str, params: &PathParams) -> Value {
        match self.new_object {
            Some(build) => build(name, params),
            None => serde_json::json!({ self.key.0: name }),
        }
    }

    #[must_use]
    pub fn label_of(&self, row: &Value) -> String {
        json::at_any(row, self.label)
            .map(|found| json::to_line(found).into_owned())
            .or_else(|| self.key.of(row).map(ToOwned::to_owned))
            .unwrap_or_else(|| "(unnamed)".to_owned())
    }

    #[must_use]
    pub fn key_of<'v>(&self, row: &'v Value) -> Option<&'v str> {
        self.key.of(row)
    }

    #[must_use]
    pub fn is_writable(&self) -> bool {
        self.ops.has(Ops::UPDATE) || self.ops.has(Ops::CREATE) || self.ops.has(Ops::DELETE)
    }

    #[must_use]
    pub const fn is_singleton(&self) -> bool {
        matches!(self.shape, Shape::Singleton)
    }

    /// Slots the *collection* request needs beyond `realm`, from the path or a fixed
    /// parameter. A non-empty list means the resource can only be listed under a parent.
    #[must_use]
    pub fn collection_slots(&self) -> Vec<&'static str> {
        let from_params = self
            .fixed_params
            .iter()
            .flat_map(|(_, value)| slots_in(value));

        self.collection
            .slots()
            .into_iter()
            .chain(from_params)
            .filter(|slot| *slot != "realm")
            .fold(Vec::new(), |mut acc, slot| {
                if !acc.contains(&slot) {
                    acc.push(slot);
                }
                acc
            })
    }

    /// The slot a row's own identifier fills in the item endpoint.
    ///
    /// It may share a name with a parent slot (`/groups/{group}/children` listed against
    /// `/groups/{group}`). That is harmless: the collection is resolved from the parent's
    /// parameters and the item from those overridden with the row's id.
    #[must_use]
    pub fn item_slot(&self) -> Option<&'static str> {
        self.item.and_then(|template| template.slots().pop())
    }

    #[must_use]
    pub fn is_nested(&self) -> bool {
        !self.collection_slots().is_empty()
    }

    /// Fixed query parameters with `{slot}` placeholders substituted.
    #[must_use]
    pub fn resolved_params(&self, params: &PathParams) -> Vec<(String, String)> {
        self.fixed_params
            .iter()
            .map(|(name, template)| ((*name).to_owned(), substitute(template, params)))
            .collect()
    }
}

fn slots_in(template: &'static str) -> impl Iterator<Item = &'static str> {
    template
        .split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}').map(|(slot, _)| slot))
}

fn substitute(template: &str, params: &PathParams) -> String {
    template
        .split('{')
        .enumerate()
        .fold(String::new(), |mut acc, (index, chunk)| {
            if index == 0 {
                acc.push_str(chunk);
                return acc;
            }
            match chunk.split_once('}') {
                Some((slot, rest)) => {
                    acc.push_str(params.get(slot).unwrap_or_default());
                    acc.push_str(rest);
                }
                None => acc.push_str(chunk),
            }
            acc
        })
}
