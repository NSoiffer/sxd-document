/// String interning using Rc<str> for safe, shared storage.
/// Replaces the previous arena-based implementation that required unsafe.
use std::borrow::Borrow;
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt, hash,
    ops::Deref,
    rc::Rc,
};

#[derive(Clone)]
pub struct InternedString(Rc<str>);

impl InternedString {
    pub fn from_str(s: &str) -> InternedString {
        InternedString(Rc::from(s))
    }

    pub fn as_slice(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &InternedString) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<str> for InternedString {
    fn eq(&self, other: &str) -> bool {
        self.as_slice().eq(other)
    }
}

impl PartialEq<&str> for InternedString {
    fn eq(&self, other: &&str) -> bool {
        self.as_slice().eq(*other)
    }
}

impl Eq for InternedString {}

impl hash::Hash for InternedString {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.0.hash(state)
    }
}

impl Borrow<str> for InternedString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for InternedString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

pub struct StringPool {
    index: RefCell<HashSet<InternedString>>,
}

impl StringPool {
    pub fn new() -> StringPool {
        StringPool {
            index: RefCell::new(HashSet::new()),
        }
    }

    pub fn intern(&self, s: &str) -> InternedString {
        if s.is_empty() {
            return InternedString::from_str("");
        }

        let mut index = self.index.borrow_mut();
        if let Some(interned) = index.get(s) {
            return interned.clone();
        }

        let interned = InternedString::from_str(s);
        index.insert(interned.clone());
        interned
    }
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;

    use super::StringPool;

    #[test]
    fn keeps_the_same_string() {
        let s = StringPool::new();

        let interned = s.intern("hello");

        assert_eq!(interned, "hello");
    }

    #[test]
    fn does_not_reuse_the_pointer_of_the_input() {
        let s = StringPool::new();
        let input = "hello";

        let interned = s.intern(input);

        assert!(input.as_bytes().as_ptr() != interned.as_bytes().as_ptr());
    }

    #[test]
    fn reuses_the_pointer_for_repeated_input() {
        let s = StringPool::new();

        let interned1 = s.intern("world");
        let interned2 = s.intern("world");

        assert_eq!(interned1.as_bytes().as_ptr(), interned2.as_bytes().as_ptr());
    }

    #[test]
    fn ignores_the_lifetime_of_the_input_string() {
        let s = StringPool::new();

        let interned = {
            let allocated_string = "green".to_owned();
            s.intern(&allocated_string)
        };

        // allocated_string is gone now, but we should be able to
        // access the result value until the storage goes away.

        assert_eq!(interned, "green");
    }

    #[test]
    fn can_be_dropped_immediately() {
        StringPool::new();
    }

    fn return_populated_storage() -> (StringPool, *const u8) {
        let s = StringPool::new();
        let ptr = {
            let interned = s.intern("hello");
            interned.as_bytes().as_ptr()
        };
        (s, ptr)
    }

    #[test]
    fn can_return_storage_populated_with_values() {
        let (s, ptr_val) = return_populated_storage();
        let interned = s.intern("hello");
        assert_eq!(interned.as_bytes().as_ptr(), ptr_val);
    }
}
