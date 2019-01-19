use crate::value::{self, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Scope<'a> {
    def: HashMap<String, Value<'a>>,
    parent: Option<ScopeHandle<'a>>,
}

pub type ScopeHandle<'a> = Rc<RefCell<Scope<'a>>>;

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Scope {
            def: HashMap::new(),
            parent: None,
        }
    }

    pub fn from(s: ScopeHandle<'a>) -> Self {
        Scope {
            def: HashMap::new(),
            parent: Some(s),
        }
    }

    pub fn to_handle(self) -> ScopeHandle<'a> {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: &str, val: Value<'a>) {
        self.def.insert(name.to_owned(), val);
    }

    pub fn assign(&mut self, name: &str, val: Value<'a>) -> Option<Value<'a>> {
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
        index: Value<'a>,
        val: Value<'a>,
    ) -> Option<Result<Value<'a>, value::Error<'a>>> {
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

    pub fn get(&self, name: &str) -> Option<Value<'a>> {
        self.def
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(name)))
    }
}
