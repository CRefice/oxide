use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Scope<'a> {
    def: HashMap<String, Value<'a>>,
    parent: Option<Rc<RefCell<Scope<'a>>>>,
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

    pub fn get(&self, name: &str) -> Option<Value<'a>> {
        self.def
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(name)))
    }
}
