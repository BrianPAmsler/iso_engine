use std::{borrow::Cow, fmt::{Debug, Display}, ops::Deref};

use crate::error::ExplicitUnwrap;


pub struct Ancestors<'a> {
    next: Option<&'a Path>
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

pub struct Path {
    inner: std::path::Path
}

impl Path {
    #[inline]
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    #[inline]
    pub fn ancestors(&self) -> Ancestors<'_> {
        Ancestors { next: Some(self) }
    }

    #[inline]
    pub fn parent(&self) -> Option<&Path> {
        self.inner.parent().map(|parent| Path::new(parent.to_str().explicit_unwrap()))
    }

    #[inline]
    pub fn to_str(&self) -> &str {
        self.inner.to_str().explicit_unwrap()
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for str {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Cow<'_, str> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl<'a> From<&'a Path> for &'a str {
    fn from(value: &'a Path) -> Self {
        value.to_str()
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_str().explicit_unwrap())
    }
}

pub struct PathBuf {
    inner: std::path::PathBuf
}

impl PathBuf {
    pub fn new() -> PathBuf {
        PathBuf { inner: std::path::PathBuf::new() }
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.inner.push(&path.as_ref().inner);
    }
}

impl<T: AsRef<str> + ?Sized> From<&T> for PathBuf {
    fn from(value: &T) -> Self {
        PathBuf::from(value.as_ref().to_string())
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        PathBuf { inner: std::path::PathBuf::from(value) }
    }
}

impl From<PathBuf> for String {
    fn from(value: PathBuf) -> Self {
        value.inner.into_os_string().into_string().explicit_unwrap()
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        Path::new(self.inner.to_str().explicit_unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{engine::resources::path::Path, error::ExplicitUnwrap};

    #[test]
    pub fn path_new() {
        let input = "test/path/test.test";

        let test = Path::new(input);
        let _t = std::path::Path::new(input);
        _t;

        let to_string = test.to_string();

        assert_eq!(&to_string, input);
    }

    #[test]
    pub fn path_ancestors() {
        let input = "test/path/test.test";

        let std_path = std::path::Path::new(input);
        let path = Path::new(input);

        let mut std_ancestors = Vec::new();
        let mut ancestors = Vec::new();

        std_path.ancestors().for_each(|ancestor| std_ancestors.push(ancestor.to_str().explicit_unwrap()));
        path.ancestors().for_each(|ancestor| ancestors.push(ancestor.to_str()));

        assert_eq!(std_ancestors, ancestors);
    }
}