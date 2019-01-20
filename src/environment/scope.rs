use crate::value::{self, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct Scope {
    def: HashMap<String, Value>,
    parent: Option<ScopeHandle>,
}

pub type ScopeHandle = Rc<RefCell<Scope>>;

impl From<Scope> for ScopeHandle {
    fn from(s: Scope) -> ScopeHandle {
        Rc::new(RefCell::new(s))
    }
}

impl Scope {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn define(&mut self, name: &str, val: Value) {
        self.def.insert(name.to_owned(), val);
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Option<Value> {
        if self.def.contains_key(name) {
            self.define(name, val.clone());
            Some(val)
        } else if let Some(s) = self.parent.clone() {
            s.borrow_mut().assign(name, val)
        } else {
            None
        }
    }

    pub fn assign_index(
        &mut self,
        name: &str,
        index: Value,
        val: Value,
    ) -> Option<Result<Value, value::Error>> {
        if let Some(v) = self.def.get_mut(name) {
            Some(v.index_mut(&index).map(|v| {
                *v = val.clone();
                val
            }))
        } else if let Some(s) = self.parent.as_mut() {
            s.borrow_mut().assign_index(name, index, val)
        } else {
            None
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.def
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(name)))
    }
}

impl<'a> From<ScopeHandle> for Scope {
    fn from(s: ScopeHandle) -> Self {
        Scope {
            def: HashMap::new(),
            parent: Some(s),
        }
    }
}
