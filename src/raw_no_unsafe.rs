use super::{lazy_hash_map::LazyHashMap, QName};

use crate::string_pool::{InternedString, StringPool};
use std::{
    cell::{Ref, RefCell},
    marker::PhantomData,
    ops::Deref,
};
use typed_generational_arena::{StandardArena as Arena, StandardIndex as Index};

// -----------------------------------------------------------------------------
// Guard types - hold RefCellReadGuard to allow safe access to arena-backed data
// -----------------------------------------------------------------------------

/// Guard for `&str` returns. Implements `Deref<Target = str>`.
/// Use `&*guard` to get `&str`.
pub struct StrGuard<'a, T> {
    _guard: Ref<'a, Arena<T>>,
    index: Index<T>,
    get_str: fn(&T) -> &str,
}

impl<T> StrGuard<'_, T> {
    pub(crate) fn new(
        guard: Ref<'_, Arena<T>>,
        index: Index<T>,
        get_str: fn(&T) -> &str,
    ) -> StrGuard<'_, T> {
        StrGuard {
            _guard: guard,
            index,
            get_str,
        }
    }
}

impl<T> Deref for StrGuard<'_, T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self._guard
            .get(self.index)
            .map(self.get_str)
            .unwrap_or("")
    }
}

impl<T> std::fmt::Display for StrGuard<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> std::fmt::Debug for StrGuard<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> PartialEq<&str> for StrGuard<'_, T> {
    fn eq(&self, other: &&str) -> bool {
        **self == **other
    }
}

impl<T> PartialEq for StrGuard<'_, T> {
    fn eq(&self, other: &StrGuard<'_, T>) -> bool {
        **self == **other
    }
}

impl<T> PartialEq<StrGuard<'_, T>> for &str {
    fn eq(&self, other: &StrGuard<'_, T>) -> bool {
        **self == **other
    }
}


/// Guard for `QName` returns. Use `.get()` to obtain the `QName`.
pub struct QNameGuard<'a, T> {
    _guard: Ref<'a, Arena<T>>,
    index: Index<T>,
    get_qname: fn(&T) -> QName<'_>,
}

impl<'a, T> QNameGuard<'a, T> {
    pub(crate) fn new(
        guard: Ref<'a, Arena<T>>,
        index: Index<T>,
        get_qname: fn(&T) -> QName<'_>,
    ) -> QNameGuard<'a, T> {
        QNameGuard {
            _guard: guard,
            index,
            get_qname,
        }
    }

    /// Get the namespace URI. The returned value borrows from this guard.
    pub fn namespace_uri(&self) -> Option<&str> {
        self._guard
            .get(self.index)
            .map(self.get_qname)
            .and_then(|q| q.namespace_uri())
    }

    /// Get the local part. The returned value borrows from this guard.
    pub fn local_part(&self) -> &str {
        self._guard
            .get(self.index)
            .map(self.get_qname)
            .map(|q| q.local_part())
            .unwrap_or("")
    }

    /// Get the QName. The returned value borrows from this guard.
    /// The caller must keep the guard alive for as long as the QName is used.
    pub fn get(&self) -> QName<'_> {
        QName::with_namespace_uri(self.namespace_uri(), self.local_part())
    }

    /// Get the namespace URI as an owned InternedString.
    /// Use this when the result needs to outlive the guard.
    pub fn namespace_uri_interned(&self) -> Option<InternedString> {
        self.namespace_uri().map(InternedString::from_str)
    }

    /// Get the local part as an owned InternedString.
    /// Use this when the result needs to outlive the guard.
    pub fn local_part_interned(&self) -> InternedString {
        InternedString::from_str(self.local_part())
    }
}

pub(crate) use typed_generational_arena::StandardIndex;

struct InternedQName {
    namespace_uri: Option<InternedString>,
    local_part: InternedString,
}

impl InternedQName {
    fn as_qname(&self) -> QName<'_> {
        QName {
            namespace_uri: self.namespace_uri.as_ref().map(|n| n.as_slice()),
            local_part: &self.local_part,
        }
    }
}

pub struct Root {
    children: Vec<ChildOfRoot>,
}

pub struct Element {
    name: InternedQName,
    default_namespace_uri: Option<InternedString>,
    preferred_prefix: Option<InternedString>,
    children: Vec<ChildOfElement>,
    parent: Option<ParentOfChild>,
    attributes: Vec<Index<Attribute>>,
    prefix_to_namespace: LazyHashMap<InternedString, InternedString>,
}

impl Element {
    pub fn name(&self) -> QName<'_> {
        self.name.as_qname()
    }
    pub fn name_local_part(&self) -> &str {
        &self.name.local_part
    }
    pub fn name_local_part_interned(&self) -> InternedString {
        self.name.local_part.clone()
    }
    pub fn default_namespace_uri(&self) -> Option<&str> {
        self.default_namespace_uri.as_ref().map(|p| p.as_slice())
    }
    pub fn preferred_prefix(&self) -> Option<&str> {
        self.preferred_prefix.as_ref().map(|p| p.as_slice())
    }
}

pub struct Attribute {
    name: InternedQName,
    preferred_prefix: Option<InternedString>,
    value: InternedString,
    parent: Option<Index<Element>>,
}

impl Attribute {
    pub fn name(&self) -> QName<'_> {
        self.name.as_qname()
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn value_interned(&self) -> InternedString {
        self.value.clone()
    }
    pub fn name_local_part_interned(&self) -> InternedString {
        self.name.local_part.clone()
    }
    pub fn name_namespace_uri_interned(&self) -> Option<InternedString> {
        self.name.namespace_uri.clone()
    }
    pub fn preferred_prefix(&self) -> Option<&str> {
        self.preferred_prefix.as_ref().map(|p| p.as_slice())
    }
}

pub struct Text {
    text: InternedString,
    parent: Option<Index<Element>>,
}

impl Text {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn text_interned(&self) -> InternedString {
        self.text.clone()
    }
}

pub struct Comment {
    text: InternedString,
    parent: Option<ParentOfChild>,
}

impl Comment {
    pub fn text(&self) -> &str {
        &self.text
    }
}

pub struct ProcessingInstruction {
    target: InternedString,
    value: Option<InternedString>,
    parent: Option<ParentOfChild>,
}

impl ProcessingInstruction {
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_ref().map(|v| v.as_slice())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChildOfRoot {
    Element(Index<Element>),
    Comment(Index<Comment>),
    ProcessingInstruction(Index<ProcessingInstruction>),
}

impl ChildOfRoot {
    fn replace_parent(&self, storage: &Storage, parent: Index<Root>) {
        let parent_id = ParentOfChild::Root(parent);
        let child_element = ChildOfElement::from(*self);
        match *self {
            ChildOfRoot::Element(n) => {
                // Root can only have one element child - remove any existing first
                let elements_to_remove: Vec<Index<Element>> = storage
                    .roots
                    .borrow()
                    .get(parent)
                    .map(|r| {
                        r.children
                            .iter()
                            .filter_map(|c| {
                                if let ChildOfRoot::Element(e) = c {
                                    Some(*e)
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                for e in &elements_to_remove {
                    if let Some(e_ref) = storage.elements.borrow_mut().get_mut(*e) {
                        e_ref.parent = None;
                    }
                }
                if let Some(r_ref) = storage.roots.borrow_mut().get_mut(parent) {
                    r_ref.children.retain(|c| !matches!(c, ChildOfRoot::Element(_)));
                }
                let prev_parent = storage.elements.borrow().get(n).and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, child_element);
                }
                if let Some(n_ref) = storage.elements.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
            ChildOfRoot::Comment(n) => {
                let prev_parent = storage.comments.borrow().get(n).and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, child_element);
                }
                if let Some(n_ref) = storage.comments.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
            ChildOfRoot::ProcessingInstruction(n) => {
                let prev_parent = storage
                    .processing_instructions
                    .borrow()
                    .get(n)
                    .and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, child_element);
                }
                if let Some(n_ref) = storage.processing_instructions.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
        };
    }

    fn remove_parent(&self, storage: &Storage) {
        match *self {
            ChildOfRoot::Element(n) => {
                if let Some(n_ref) = storage.elements.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
            ChildOfRoot::Comment(n) => {
                if let Some(n_ref) = storage.comments.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
            ChildOfRoot::ProcessingInstruction(n) => {
                if let Some(n_ref) = storage.processing_instructions.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChildOfElement {
    Element(Index<Element>),
    Text(Index<Text>),
    Comment(Index<Comment>),
    ProcessingInstruction(Index<ProcessingInstruction>),
}

fn child_of_element_to_root(c: ChildOfElement) -> Option<ChildOfRoot> {
    match c {
        ChildOfElement::Element(n) => Some(ChildOfRoot::Element(n)),
        ChildOfElement::Comment(n) => Some(ChildOfRoot::Comment(n)),
        ChildOfElement::ProcessingInstruction(n) => Some(ChildOfRoot::ProcessingInstruction(n)),
        ChildOfElement::Text(_) => None, // Text cannot be child of Root
    }
}

fn remove_child_from_prev_parent(
    storage: &Storage,
    prev_parent: ParentOfChild,
    child: ChildOfElement,
) {
    match prev_parent {
        ParentOfChild::Root(r) => {
            if let Some(child_root) = child_of_element_to_root(child) {
                if let Some(r_ref) = storage.roots.borrow_mut().get_mut(r) {
                    r_ref.children.retain(|n| *n != child_root);
                }
            }
        }
        ParentOfChild::Element(e) => {
            if let Some(e_ref) = storage.elements.borrow_mut().get_mut(e) {
                e_ref.children.retain(|n| *n != child);
            }
        }
    }
}

impl ChildOfElement {
    fn replace_parent(&self, storage: &Storage, parent: Index<Element>) {
        let parent_id = ParentOfChild::Element(parent);
        match *self {
            ChildOfElement::Element(n) => {
                let prev_parent = storage.elements.borrow().get(n).and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, *self);
                }
                if let Some(n_ref) = storage.elements.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
            ChildOfElement::Comment(n) => {
                let prev_parent = storage.comments.borrow().get(n).and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, *self);
                }
                if let Some(n_ref) = storage.comments.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
            ChildOfElement::ProcessingInstruction(n) => {
                let prev_parent = storage
                    .processing_instructions
                    .borrow()
                    .get(n)
                    .and_then(|r| r.parent);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, *self);
                }
                if let Some(n_ref) = storage.processing_instructions.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent_id);
                }
            }
            ChildOfElement::Text(n) => {
                let prev_parent = storage
                    .texts
                    .borrow()
                    .get(n)
                    .and_then(|r| r.parent)
                    .map(ParentOfChild::Element);
                if let Some(prev) = prev_parent {
                    remove_child_from_prev_parent(storage, prev, *self);
                }
                if let Some(n_ref) = storage.texts.borrow_mut().get_mut(n) {
                    n_ref.parent = Some(parent);
                }
            }
        };
    }

    fn remove_parent(&self, storage: &Storage) {
        match *self {
            ChildOfElement::Element(n) => {
                if let Some(n_ref) = storage.elements.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
            ChildOfElement::Comment(n) => {
                if let Some(n_ref) = storage.comments.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
            ChildOfElement::ProcessingInstruction(n) => {
                if let Some(n_ref) = storage.processing_instructions.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
            ChildOfElement::Text(n) => {
                if let Some(n_ref) = storage.texts.borrow_mut().get_mut(n) {
                    n_ref.parent = None;
                }
            }
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParentOfChild {
    Root(Index<Root>),
    Element(Index<Element>),
}

macro_rules! conversion_trait(
    ($res_type:ident, {
        $($leaf_type:ident => $variant:expr),*
    }) => (
        $(impl From<Index<$leaf_type>> for $res_type {
            fn from(v: Index<$leaf_type>) -> $res_type {
                $variant(v)
            }
        })*
    )
);

conversion_trait!(
    ChildOfElement, {
        Element               => ChildOfElement::Element,
        Text                  => ChildOfElement::Text,
        Comment               => ChildOfElement::Comment,
        ProcessingInstruction => ChildOfElement::ProcessingInstruction
    }
);

conversion_trait!(
    ChildOfRoot, {
        Element               => ChildOfRoot::Element,
        Comment               => ChildOfRoot::Comment,
        ProcessingInstruction => ChildOfRoot::ProcessingInstruction
    }
);

impl From<ChildOfRoot> for ChildOfElement {
    fn from(v: ChildOfRoot) -> ChildOfElement {
        match v {
            ChildOfRoot::Element(n) => ChildOfElement::Element(n),
            ChildOfRoot::Comment(n) => ChildOfElement::Comment(n),
            ChildOfRoot::ProcessingInstruction(n) => ChildOfElement::ProcessingInstruction(n),
        }
    }
}

pub struct Storage {
    pub(crate) strings: StringPool,
    pub(crate) roots: RefCell<Arena<Root>>,
    pub(crate) elements: RefCell<Arena<Element>>,
    pub(crate) attributes: RefCell<Arena<Attribute>>,
    pub(crate) texts: RefCell<Arena<Text>>,
    pub(crate) comments: RefCell<Arena<Comment>>,
    pub(crate) processing_instructions: RefCell<Arena<ProcessingInstruction>>,
}

impl Default for Storage {
    fn default() -> Storage {
        Storage {
            strings: StringPool::new(),
            roots: RefCell::new(Arena::new()),
            elements: RefCell::new(Arena::new()),
            attributes: RefCell::new(Arena::new()),
            texts: RefCell::new(Arena::new()),
            comments: RefCell::new(Arena::new()),
            processing_instructions: RefCell::new(Arena::new()),
        }
    }
}

impl Storage {
    pub fn new() -> Storage {
        Self::default()
    }

    fn intern(&self, s: &str) -> InternedString {
        self.strings.intern(s)
    }

    fn intern_qname(&self, q: QName<'_>) -> InternedQName {
        InternedQName {
            namespace_uri: q.namespace_uri.map(|p| self.intern(p)),
            local_part: self.intern(q.local_part),
        }
    }

    pub fn create_root(&self) -> Index<Root> {
        self.roots.borrow_mut().insert(Root {
            children: Vec::new(),
        })
    }

    pub fn create_element<'n, N>(&self, name: N) -> Index<Element>
    where
        N: Into<QName<'n>>,
    {
        let name = name.into();
        let name = self.intern_qname(name);

        self.elements.borrow_mut().insert(Element {
            name,
            default_namespace_uri: None,
            preferred_prefix: None,
            children: Vec::new(),
            parent: None,
            attributes: Vec::new(),
            prefix_to_namespace: LazyHashMap::new(),
        })
    }

    pub fn create_attribute<'n, N>(&self, name: N, value: &str) -> Index<Attribute>
    where
        N: Into<QName<'n>>,
    {
        let name = name.into();
        let name = self.intern_qname(name);
        let value = self.intern(value);

        self.attributes.borrow_mut().insert(Attribute {
            name,
            preferred_prefix: None,
            value,
            parent: None,
        })
    }

    pub fn create_text(&self, text: &str) -> Index<Text> {
        let text = self.intern(text);

        self.texts.borrow_mut().insert(Text { text, parent: None })
    }

    pub fn create_comment(&self, text: &str) -> Index<Comment> {
        let text = self.intern(text);

        self.comments.borrow_mut().insert(Comment { text, parent: None })
    }

    pub fn create_processing_instruction(
        &self,
        target: &str,
        value: Option<&str>,
    ) -> Index<ProcessingInstruction> {
        let target = self.intern(target);
        let value = value.map(|v| self.intern(v));

        self.processing_instructions
            .borrow_mut()
            .insert(ProcessingInstruction {
                target,
                value,
                parent: None,
            })
    }

    pub fn element_set_name<'n, N>(&self, element: Index<Element>, name: N)
    where
        N: Into<QName<'n>>,
    {
        let name = name.into();
        let name = self.intern_qname(name);
        if let Some(element_r) = self.elements.borrow_mut().get_mut(element) {
            element_r.name = name;
        }
    }

    pub fn element_register_prefix(
        &self,
        element: Index<Element>,
        prefix: &str,
        namespace_uri: &str,
    ) {
        let prefix = self.intern(prefix);
        let namespace_uri = self.intern(namespace_uri);
        if let Some(element_r) = self.elements.borrow_mut().get_mut(element) {
            element_r.prefix_to_namespace.insert(prefix, namespace_uri);
        }
    }

    pub fn element_set_default_namespace_uri(
        &self,
        element: Index<Element>,
        namespace_uri: Option<&str>,
    ) {
        let namespace_uri = namespace_uri.map(|p| self.intern(p));
        if let Some(element_r) = self.elements.borrow_mut().get_mut(element) {
            element_r.default_namespace_uri = namespace_uri;
        }
    }

    pub fn element_set_preferred_prefix(&self, element: Index<Element>, prefix: Option<&str>) {
        let prefix = prefix.map(|p| self.intern(p));
        if let Some(element_r) = self.elements.borrow_mut().get_mut(element) {
            element_r.preferred_prefix = prefix;
        }
    }

    pub fn attribute_set_preferred_prefix(&self, attribute: Index<Attribute>, prefix: Option<&str>) {
        let prefix = prefix.map(|p| self.intern(p));
        if let Some(attribute_r) = self.attributes.borrow_mut().get_mut(attribute) {
            attribute_r.preferred_prefix = prefix;
        }
    }

    pub fn text_set_text(&self, text: Index<Text>, new_text: &str) {
        let new_text = self.intern(new_text);
        if let Some(text_r) = self.texts.borrow_mut().get_mut(text) {
            text_r.text = new_text;
        }
    }

    pub fn comment_set_text(&self, comment: Index<Comment>, new_text: &str) {
        let new_text = self.intern(new_text);
        if let Some(comment_r) = self.comments.borrow_mut().get_mut(comment) {
            comment_r.text = new_text;
        }
    }

    pub fn processing_instruction_set_target(
        &self,
        pi: Index<ProcessingInstruction>,
        new_target: &str,
    ) {
        let new_target = self.intern(new_target);
        if let Some(pi_r) = self.processing_instructions.borrow_mut().get_mut(pi) {
            pi_r.target = new_target;
        }
    }

    pub fn processing_instruction_set_value(
        &self,
        pi: Index<ProcessingInstruction>,
        new_value: Option<&str>,
    ) {
        let new_value = new_value.map(|v| self.intern(v));
        if let Some(pi_r) = self.processing_instructions.borrow_mut().get_mut(pi) {
            pi_r.value = new_value;
        }
    }
}

pub struct Connections {
    root: Index<Root>,
}

impl Connections {
    pub fn new(root: Index<Root>) -> Connections {
        Connections { root }
    }

    pub fn root(&self) -> Index<Root> {
        self.root
    }

    pub fn element_parent(&self, storage: &Storage, child: Index<Element>) -> Option<ParentOfChild> {
        storage.elements.borrow().get(child).and_then(|c| c.parent)
    }

    pub fn text_parent(&self, storage: &Storage, child: Index<Text>) -> Option<Index<Element>> {
        storage.texts.borrow().get(child).and_then(|c| c.parent)
    }

    pub fn comment_parent(
        &self,
        storage: &Storage,
        child: Index<Comment>,
    ) -> Option<ParentOfChild> {
        storage.comments.borrow().get(child).and_then(|c| c.parent)
    }

    pub fn processing_instruction_parent(
        &self,
        storage: &Storage,
        child: Index<ProcessingInstruction>,
    ) -> Option<ParentOfChild> {
        storage
            .processing_instructions
            .borrow()
            .get(child)
            .and_then(|c| c.parent)
    }

    pub fn append_root_child<C>(&self, storage: &Storage, child: C)
    where
        C: Into<ChildOfRoot>,
    {
        let child = child.into();
        child.replace_parent(storage, self.root);

        if let Some(parent_r) = storage.roots.borrow_mut().get_mut(self.root) {
            parent_r.children.push(child);
        }
    }

    pub fn append_element_child<C>(&self, storage: &Storage, parent: Index<Element>, child: C)
    where
        C: Into<ChildOfElement>,
    {
        let child = child.into();
        child.replace_parent(storage, parent);

        if let Some(parent_r) = storage.elements.borrow_mut().get_mut(parent) {
            parent_r.children.push(child);
        }
    }

    pub fn remove_root_child<C>(&self, storage: &Storage, child: C)
    where
        C: Into<ChildOfRoot>,
    {
        let child = child.into();
        child.remove_parent(storage);

        if let Some(parent_r) = storage.roots.borrow_mut().get_mut(self.root) {
            parent_r.children.retain(|&x| x != child);
        }
    }

    pub fn remove_element_child<C>(&self, storage: &Storage, parent: Index<Element>, child: C)
    where
        C: Into<ChildOfElement>,
    {
        let child = child.into();
        child.remove_parent(storage);

        if let Some(parent_r) = storage.elements.borrow_mut().get_mut(parent) {
            parent_r.children.retain(|&x| x != child);
        }
    }

    pub fn clear_root_children(&self, storage: &Storage) {
        let children: Vec<ChildOfRoot> = storage
            .roots
            .borrow()
            .get(self.root)
            .map(|r| r.children.clone())
            .unwrap_or_default();
        for c in children {
            c.remove_parent(storage);
        }
        if let Some(parent_r) = storage.roots.borrow_mut().get_mut(self.root) {
            parent_r.children.clear();
        }
    }

    pub fn clear_element_children(&self, storage: &Storage, parent: Index<Element>) {
        let children: Vec<ChildOfElement> = storage
            .elements
            .borrow()
            .get(parent)
            .map(|e| e.children.clone())
            .unwrap_or_default();
        for c in children {
            c.remove_parent(storage);
        }
        if let Some(parent_r) = storage.elements.borrow_mut().get_mut(parent) {
            parent_r.children.clear();
        }
    }

    pub fn remove_element_from_parent(&self, storage: &Storage, child: Index<Element>) {
        let parent = storage.elements.borrow().get(child).and_then(|child_r| child_r.parent);
        match parent {
            Some(ParentOfChild::Root(_)) => self.remove_root_child(storage, ChildOfRoot::Element(child)),
            Some(ParentOfChild::Element(parent)) => {
                self.remove_element_child(storage, parent, ChildOfElement::Element(child))
            }
            None => { /* no-op */ }
        }
    }

    pub fn remove_attribute_from_parent(&self, storage: &Storage, child: Index<Attribute>) {
        let parent = storage.attributes.borrow().get(child).and_then(|child_r| child_r.parent);
        if let Some(parent) = parent {
            self.remove_attribute_x(storage, parent, |_, attr| attr == child);
        }
    }

    pub fn remove_text_from_parent(&self, storage: &Storage, child: Index<Text>) {
        let parent = storage.texts.borrow().get(child).and_then(|child_r| child_r.parent);
        if let Some(parent) = parent {
            self.remove_element_child(storage, parent, ChildOfElement::Text(child));
        }
    }

    pub fn remove_comment_from_parent(&self, storage: &Storage, child: Index<Comment>) {
        let parent = storage.comments.borrow().get(child).and_then(|child_r| child_r.parent);
        match parent {
            Some(ParentOfChild::Root(_)) => self.remove_root_child(storage, ChildOfRoot::Comment(child)),
            Some(ParentOfChild::Element(parent)) => {
                self.remove_element_child(storage, parent, ChildOfElement::Comment(child))
            }
            None => { /* no-op */ }
        }
    }

    pub fn remove_processing_instruction_from_parent(
        &self,
        storage: &Storage,
        child: Index<ProcessingInstruction>,
    ) {
        let parent = storage
            .processing_instructions
            .borrow()
            .get(child)
            .and_then(|child_r| child_r.parent);
        match parent {
            Some(ParentOfChild::Root(_)) => self.remove_root_child(
                storage,
                ChildOfRoot::ProcessingInstruction(child),
            ),
            Some(ParentOfChild::Element(parent)) => self.remove_element_child(
                storage,
                parent,
                ChildOfElement::ProcessingInstruction(child),
            ),
            None => { /* no-op */ }
        }
    }

    pub fn root_children(&self, storage: &Storage) -> Vec<ChildOfRoot> {
        storage
            .roots
            .borrow()
            .get(self.root)
            .map(|r| r.children.clone())
            .unwrap_or_default()
    }

    pub fn element_children(&self, storage: &Storage, parent: Index<Element>) -> Vec<ChildOfElement> {
        storage
            .elements
            .borrow()
            .get(parent)
            .map(|e| e.children.clone())
            .unwrap_or_default()
    }

    /// Returns the sibling nodes that come before this node. The
    /// nodes are in document order.
    pub fn element_preceding_siblings(
        &self,
        storage: &Storage,
        element: Index<Element>,
    ) -> SiblingIter {
        if let Some(element_r) = storage.elements.borrow().get(element) {
            match element_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Preceding,
                    root_parent,
                    ChildOfRoot::Element(element),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Preceding,
                    element_parent,
                    ChildOfElement::Element(element),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come after this node. The
    /// nodes are in document order.
    pub fn element_following_siblings(
        &self,
        storage: &Storage,
        element: Index<Element>,
    ) -> SiblingIter {
        if let Some(element_r) = storage.elements.borrow().get(element) {
            match element_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Following,
                    root_parent,
                    ChildOfRoot::Element(element),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Following,
                    element_parent,
                    ChildOfElement::Element(element),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come before this node. The
    /// nodes are in document order.
    pub fn text_preceding_siblings(&self, storage: &Storage, text: Index<Text>) -> SiblingIter {
        if let Some(text_r) = storage.texts.borrow().get(text) {
            match text_r.parent {
                Some(element_parent) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Preceding,
                    element_parent,
                    ChildOfElement::Text(text),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come after this node. The
    /// nodes are in document order.
    pub fn text_following_siblings(&self, storage: &Storage, text: Index<Text>) -> SiblingIter {
        if let Some(text_r) = storage.texts.borrow().get(text) {
            match text_r.parent {
                Some(element_parent) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Following,
                    element_parent,
                    ChildOfElement::Text(text),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come before this node. The
    /// nodes are in document order.
    pub fn comment_preceding_siblings(
        &self,
        storage: &Storage,
        comment: Index<Comment>,
    ) -> SiblingIter {
        if let Some(comment_r) = storage.comments.borrow().get(comment) {
            match comment_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Preceding,
                    root_parent,
                    ChildOfRoot::Comment(comment),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Preceding,
                    element_parent,
                    ChildOfElement::Comment(comment),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come after this node. The
    /// nodes are in document order.
    pub fn comment_following_siblings(
        &self,
        storage: &Storage,
        comment: Index<Comment>,
    ) -> SiblingIter {
        if let Some(comment_r) = storage.comments.borrow().get(comment) {
            match comment_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Following,
                    root_parent,
                    ChildOfRoot::Comment(comment),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Following,
                    element_parent,
                    ChildOfElement::Comment(comment),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come before this node. The
    /// nodes are in document order.
    pub fn processing_instruction_preceding_siblings(
        &self,
        storage: &Storage,
        pi: Index<ProcessingInstruction>,
    ) -> SiblingIter {
        if let Some(pi_r) = storage.processing_instructions.borrow().get(pi) {
            match pi_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Preceding,
                    root_parent,
                    ChildOfRoot::ProcessingInstruction(pi),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Preceding,
                    element_parent,
                    ChildOfElement::ProcessingInstruction(pi),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    /// Returns the sibling nodes that come after this node. The
    /// nodes are in document order.
    pub fn processing_instruction_following_siblings(
        &self,
        storage: &Storage,
        pi: Index<ProcessingInstruction>,
    ) -> SiblingIter {
        if let Some(pi_r) = storage.processing_instructions.borrow().get(pi) {
            match pi_r.parent {
                Some(ParentOfChild::Root(root_parent)) => SiblingIter::of_root(
                    storage,
                    SiblingDirection::Following,
                    root_parent,
                    ChildOfRoot::ProcessingInstruction(pi),
                ),
                Some(ParentOfChild::Element(element_parent)) => SiblingIter::of_element(
                    storage,
                    SiblingDirection::Following,
                    element_parent,
                    ChildOfElement::ProcessingInstruction(pi),
                ),
                None => SiblingIter::dead(),
            }
        } else {
            SiblingIter::dead()
        }
    }

    pub fn attribute_parent(&self, storage: &Storage, attribute: Index<Attribute>) -> Option<Index<Element>> {
        storage.attributes.borrow().get(attribute).and_then(|a| a.parent)
    }

    pub fn attributes(&self, storage: &Storage, parent: Index<Element>) -> Vec<Index<Attribute>> {
        storage
            .elements
            .borrow()
            .get(parent)
            .map(|e| e.attributes.clone())
            .unwrap_or_default()
    }

    pub fn attribute<'n, N>(&self, storage: &Storage, element: Index<Element>, name: N) -> Option<Index<Attribute>>
    where
        N: Into<QName<'n>>,
    {
        let name = name.into();
        storage
            .elements
            .borrow()
            .get(element)
            .and_then(|e| {
                e.attributes.iter().find(|a| {
                    storage
                        .attributes
                        .borrow()
                        .get(**a)
                        .map(|a_r| a_r.name.as_qname() == name)
                        .unwrap_or(false)
                }).copied()
            })
    }

    pub fn remove_attribute<'n, N>(&self, storage: &Storage, element: Index<Element>, name: N)
    where
        N: Into<QName<'n>>,
    {
        let name = name.into();
        self.remove_attribute_x(storage, element, |s, a| {
            s.attributes.borrow().get(a).map(|ar| ar.name.as_qname() == name).unwrap_or(false)
        })
    }

    pub fn remove_attribute_x<F>(&self, storage: &Storage, element: Index<Element>, mut pred: F)
    where
        F: FnMut(&Storage, Index<Attribute>) -> bool,
    {
        if let Some(element_r) = storage.elements.borrow_mut().get_mut(element) {
            let attrs = std::mem::take(&mut element_r.attributes);
            for a in attrs {
                let is_this_attr = pred(storage, a);
                if is_this_attr {
                    if let Some(a_r) = storage.attributes.borrow_mut().get_mut(a) {
                        a_r.parent = None;
                    }
                } else {
                    element_r.attributes.push(a);
                }
            }
        }
    }

    pub fn set_attribute(&self, storage: &Storage, parent: Index<Element>, attribute: Index<Attribute>) {
        let attr_data = storage
            .attributes
            .borrow()
            .get(attribute)
            .map(|a| (a.name.namespace_uri.clone(), a.name.local_part.clone()));
        if let Some((ns_uri, local_part)) = attr_data {
            let attr_qname = QName::with_namespace_uri(
                ns_uri.as_ref().map(|s| s.as_slice()),
                local_part.as_slice(),
            );
            if let Some(prev_parent) = storage.attributes.borrow().get(attribute).and_then(|a| a.parent) {
                if let Some(prev_parent_r) = storage.elements.borrow_mut().get_mut(prev_parent) {
                    prev_parent_r.attributes.retain(|&a| a != attribute);
                }
            }

            if let Some(parent_r) = storage.elements.borrow_mut().get_mut(parent) {
                parent_r.attributes.retain(|a| {
                    storage
                        .attributes
                        .borrow()
                        .get(*a)
                        .map(|a_r| a_r.name.as_qname() != attr_qname)
                        .unwrap_or(true)
                });
                parent_r.attributes.push(attribute);
            }
            if let Some(attr_r) = storage.attributes.borrow_mut().get_mut(attribute) {
                attr_r.parent = Some(parent);
            }
        }
    }

    fn element_parents<'a>(&self, storage: &'a Storage, element: Index<Element>) -> ElementParents<'a> {
        ElementParents {
            storage,
            element: Some(element),
            marker: PhantomData,
        }
    }

    pub fn element_namespace_uri_for_prefix(
        &self,
        storage: &Storage,
        element: Index<Element>,
        prefix: &str,
    ) -> Option<InternedString> {
        for eid in self.element_parents(storage, element) {
            if let Some(e) = storage.elements.borrow().get(eid) {
                if let Some(ns_uri) = e.prefix_to_namespace.get(prefix) {
                    return Some(ns_uri.clone());
                }
            }
        }
        None
    }

    pub fn element_prefix_for_namespace_uri(
        &self,
        storage: &Storage,
        element: Index<Element>,
        namespace_uri: &str,
        preferred_prefix: Option<&str>,
    ) -> Option<InternedString> {
        for eid in self.element_parents(storage, element) {
            let elements_borrow = storage.elements.borrow();
            let element_r = match elements_borrow.get(eid) {
                Some(e) => e,
                None => continue,
            };
            let mut matching_prefix: Option<InternedString> = None;
            for (prefix, ns_uri) in element_r.prefix_to_namespace.iter() {
                if ns_uri.as_slice() == namespace_uri {
                    if let Some(preferred) = preferred_prefix {
                        if prefix.as_slice() == preferred {
                            let result = prefix.clone();
                            drop(elements_borrow);
                            return Some(result);
                        }
                    }
                    if matching_prefix.is_none() {
                        matching_prefix = Some(prefix.clone());
                    }
                }
            }
            drop(elements_borrow);
            if let Some(prefix) = matching_prefix {
                return Some(prefix);
            }
        }
        None
    }

    pub fn element_namespaces_in_scope<'a>(
        &self,
        storage: &'a Storage,
        element: Index<Element>,
    ) -> NamespacesInScope<'a> {
        let element_ids: Vec<_> = self.element_parents(storage, element).collect();
        let mut namespaces: Vec<(std::string::String, std::string::String)> = Vec::new();

        namespaces.push((
            crate::XML_NS_PREFIX.to_string(),
            crate::XML_NS_URI.to_string(),
        ));

        let guard = storage.elements.borrow();
        for eid in element_ids {
            if let Some(element_r) = guard.get(eid) {
                for (prefix, uri) in element_r.prefix_to_namespace.iter() {
                    let prefix_s = prefix.as_slice().to_string();
                    let uri_s = uri.as_slice().to_string();
                    if !namespaces.iter().any(|ns| ns.0 == prefix_s) {
                        namespaces.push((prefix_s, uri_s))
                    }
                }
            }
        }

        NamespacesInScope {
            data: namespaces,
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn element_default_namespace_uri(
        &self,
        storage: &Storage,
        element: Index<Element>,
    ) -> Option<InternedString> {
        for eid in self.element_parents(storage, element) {
            if let Some(e) = storage.elements.borrow().get(eid) {
                if let Some(ns) = &e.default_namespace_uri {
                    return Some(ns.clone());
                }
            }
        }
        None
    }
}

struct ElementParents<'a> {
    storage: &'a Storage,
    element: Option<Index<Element>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for ElementParents<'a> {
    type Item = Index<Element>;

    fn next(&mut self) -> Option<Index<Element>> {
        let element_id = self.element?;
        let elements_borrow = self.storage.elements.borrow();
        let element_ref = elements_borrow.get(element_id)?;
        let next_element = match element_ref.parent {
            Some(ParentOfChild::Element(parent)) => Some(parent),
            _ => None,
        };
        drop(elements_borrow);

        self.element = next_element;
        Some(element_id)
    }
}

pub struct NamespacesInScope<'a> {
    data: Vec<(std::string::String, std::string::String)>,
    index: usize,
    _marker: PhantomData<&'a ()>,
}

impl Iterator for NamespacesInScope<'_> {
    type Item = (std::string::String, std::string::String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = self.data[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

enum SiblingDirection {
    Preceding,
    Following,
}

pub struct SiblingIter {
    siblings: Vec<ChildOfElement>,
    index: usize,
}

impl SiblingIter {
    fn of_root(
        storage: &Storage,
        direction: SiblingDirection,
        root_parent: Index<Root>,
        child: ChildOfRoot,
    ) -> SiblingIter {
        let siblings = storage
            .roots
            .borrow()
            .get(root_parent)
            .map(|r| {
                let data = &r.children;
                let pos = data.iter().position(|c| *c == child).unwrap_or(0);
                let slice = match direction {
                    SiblingDirection::Preceding => &data[..pos],
                    SiblingDirection::Following => &data[pos + 1..],
                };
                slice.iter().map(|c| (*c).into()).collect()
            })
            .unwrap_or_default();
        SiblingIter {
            siblings,
            index: 0,
        }
    }

    fn of_element(
        storage: &Storage,
        direction: SiblingDirection,
        element_parent: Index<Element>,
        child: ChildOfElement,
    ) -> SiblingIter {
        let siblings = storage
            .elements
            .borrow()
            .get(element_parent)
            .map(|e| {
                let data = &e.children;
                let pos = data.iter().position(|c| *c == child).unwrap_or(0);
                let slice = match direction {
                    SiblingDirection::Preceding => &data[..pos],
                    SiblingDirection::Following => &data[pos + 1..],
                };
                slice.to_vec()
            })
            .unwrap_or_default();
        SiblingIter {
            siblings,
            index: 0,
        }
    }

    fn dead() -> SiblingIter {
        SiblingIter {
            siblings: Vec::new(),
            index: 0,
        }
    }
}

impl Iterator for SiblingIter {
    type Item = ChildOfElement;

    fn next(&mut self) -> Option<ChildOfElement> {
        if self.index < self.siblings.len() {
            let item = self.siblings[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
