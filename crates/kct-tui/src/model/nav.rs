use std::sync::Arc;

use kct_core::Context;
use kct_core::catalog::Resource;

/// One level of the navigation stack.
#[derive(Debug, Clone)]
pub struct Location {
    pub resource: &'static Resource,
    pub context: Context,
    /// What to show in the breadcrumb — the parent object's name, for nested levels.
    pub crumb: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct Nav {
    /// Never empty; the last element is what is on screen.
    stack: Vec<Location>,
    pub sidebar_cursor: usize,
    pub sidebar_focused: bool,
}

impl Nav {
    #[must_use]
    pub fn rooted(resource: &'static Resource, context: Context) -> Self {
        Self {
            stack: vec![Location {
                resource,
                context,
                crumb: Arc::from(resource.plural),
            }],
            sidebar_cursor: 0,
            sidebar_focused: false,
        }
    }

    #[must_use]
    pub fn with_sidebar_focus(self, sidebar_focused: bool) -> Self {
        Self {
            sidebar_focused,
            ..self
        }
    }

    #[must_use]
    pub fn with_sidebar_cursor(self, sidebar_cursor: usize) -> Self {
        Self {
            sidebar_cursor,
            ..self
        }
    }

    #[must_use]
    pub fn current(&self) -> &Location {
        self.stack
            .last()
            .expect("the navigation stack is never empty")
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    #[must_use]
    pub fn crumbs(&self) -> Vec<&str> {
        self.stack.iter().map(|location| &*location.crumb).collect()
    }

    #[must_use]
    pub fn pushed(self, location: Location) -> Self {
        let mut stack = self.stack;
        stack.push(location);
        Self { stack, ..self }
    }

    /// Leave the current level. Returns `None` at the root, where "back" means nothing.
    #[must_use]
    pub fn popped(self) -> Option<Self> {
        let mut stack = self.stack;
        (stack.len() > 1).then(|| {
            stack.pop();
            Self { stack, ..self }
        })
    }

    /// Replace the whole stack — used when jumping to a screen from the palette.
    #[must_use]
    pub fn jumped(self, resource: &'static Resource, context: Context) -> Self {
        Self {
            stack: vec![Location {
                resource,
                context,
                crumb: Arc::from(resource.plural),
            }],
            ..self
        }
    }

    /// Re-root every level at a new realm, keeping the parent chain meaningless but the
    /// screen the operator was on.
    #[must_use]
    pub fn rerooted(self, resource: &'static Resource, context: Context) -> Self {
        self.jumped(resource, context)
    }
}
