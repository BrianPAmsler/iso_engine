use std::{borrow::{Borrow, Cow}, collections::TryReserveError, ffi::OsStr, fmt::{Debug, Display}, iter::FusedIterator, ops::Deref, rc::Rc, sync::Arc};

use delegate::delegate;

use crate::error::ExplicitUnwrap;

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Path {
    inner: std::path::Path
}

impl Path {
    fn from_inner(inner: &std::path::Path) -> &Self {
        unsafe { &*(inner as *const std::path::Path as *const Path) }
    }

    fn from_inner_mut(inner: &mut std::path::Path) -> &mut Self {
        unsafe { &mut *(inner as *mut std::path::Path as *mut Path) }
    }

    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        Self::from_inner(std::path::Path::new(s.as_ref()))
    }

    pub fn strip_prefix<P: AsRef<Path>>(&self, base: P) -> Result<&Path, std::path::StripPrefixError> {
        self.inner.strip_prefix(&base.as_ref().inner).map(Self::from_inner)
    }

    pub fn starts_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self.inner.starts_with(&child.as_ref().inner)
    }

    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self.inner.ends_with(&child.as_ref().inner)
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        PathBuf { inner: self.inner.join(&path.as_ref().inner) }
    }

    pub fn with_file_name<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        PathBuf { inner: self.inner.with_file_name(file_name.as_ref()) }
    }

    pub fn with_extension<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        PathBuf { inner: self.inner.with_extension(file_name.as_ref()) }
    }

    pub fn with_added_extension<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        PathBuf { inner: self.inner.with_added_extension(file_name.as_ref()) }
    }

    pub fn into_path_buf(self: Box<Self>) -> PathBuf {
        let inner: Box<std::path::Path> = unsafe { std::mem::transmute(self) };
        PathBuf { inner: inner.into_path_buf() }
    }

    delegate! {
        to self.inner {
            #[unwrap]
            pub fn to_str(&self) -> &str;

            #[expr(PathBuf { inner: $ })]
            pub fn to_path_buf(&self) -> PathBuf;

            #[expr($.map(Self::from_inner))]
            pub fn parent(&self) -> Option<&Path>;

            #[expr(Ancestors{ inner: $ })]
            pub fn ancestors(&self) -> Ancestors<'_>;

            #[expr($.map(|s| s.to_str().unwrap()))]
            pub fn file_name(&self) -> Option<&str>;

            #[expr($.map(|s| s.to_str().unwrap()))]
            pub fn file_stem(&self) -> Option<&str>;

            #[expr($.map(|s| s.to_str().unwrap()))]
            pub fn file_prefix(&self) -> Option<&str>;

            #[expr($.map(|s| s.to_str().unwrap()))]
            pub fn extension(&self) -> Option<&str>;

            #[expr(Components { inner: $ })]
            pub fn components(&self) -> Components<'_>;

            #[expr(Iter { inner: $})]
            pub fn iter(&self) -> Iter<'_>;
        }
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

impl<'a> From<&'a Path> for &'a str {
    fn from(value: &'a Path) -> Self {
        value.to_str()
    }
}

impl<'a> From<&'a Path> for Cow<'a, Path> {
    fn from(value: &'a Path) -> Self {
        let cow: Cow<'a, std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(cow) }
    }
}

impl From<&Path> for Rc<Path> {
    fn from(value: &Path) -> Self {
        let rc: Rc<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(rc) }
    }
}

impl From<&Path> for Arc<Path> {
    fn from(value: &Path) -> Self {
        let arc: Arc<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(arc) }
    }
}

impl From<&Path> for Box<Path> {
    fn from(value: &Path) -> Self {
        let boxed: Box<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(boxed) }
    }
}

impl<'a> From<&'a mut Path> for Cow<'a, Path> {
    fn from(value: &'a mut Path) -> Self {
        let cow: Cow<'a, std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(cow) }
    }
}

impl From<&mut Path> for Rc<Path> {
    fn from(value: &mut Path) -> Self {
        let rc: Rc<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(rc) }
    }
}

impl From<&mut Path> for Arc<Path> {
    fn from(value: &mut Path) -> Self {
        let arc: Arc<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(arc) }
    }
}

impl From<&mut Path> for Box<Path> {
    fn from(value: &mut Path) -> Self {
        let boxed: Box<std::path::Path> = value.inner.into();

        unsafe { std::mem::transmute(boxed) }
    }
}

impl From<PathBuf> for Box<Path> {
    fn from(value: PathBuf) -> Self {
        unsafe { std::mem::transmute(value.inner.into_boxed_path()) }
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;
    
    fn to_owned(&self) -> Self::Owned {
        self.to_path_buf()
    }

}

impl PartialEq<str> for Path {
    fn eq(&self, other: &str) -> bool {
        self.inner.eq(other)
    }
}

impl Clone for Box<Path> {
    #[allow(clippy::borrowed_box)]
    #[inline]
    fn clone(&self) -> Self {
        let boxed: &Box<std::path::Path> = unsafe { std::mem::transmute(&self) };
        let boxed = boxed.clone();
        unsafe { std::mem::transmute(boxed) }
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

#[derive(Clone, Copy, Debug)]
pub struct Ancestors<'a> {
    inner: std::path::Ancestors<'a>
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Path::from_inner)
    }
}

impl FusedIterator for Ancestors<'_> {}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Components<'a> {
    inner: std::path::Components<'a>
}

impl Components<'_> {
    pub fn as_path(&self) -> &Path {
        Path::from_inner(self.inner.as_path())
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Component::from_inner)
    }
}

impl DoubleEndedIterator for Components<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Component::from_inner)
    }
}

impl FusedIterator for Components<'_> {}

impl AsRef<str> for Components<'_> {
    fn as_ref(&self) -> &str {
        let os_str: &OsStr = self.inner.as_ref();
        os_str.to_str().explicit_unwrap()
    }
}

impl AsRef<Path> for Components<'_> {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Component<'a> {
    inner: std::path::Component<'a>
}

impl<'a> Component<'a> {
    fn from_inner(inner: std::path::Component<'_>) -> Component<'_> {
        Component { inner }
    }

    delegate! {
        to self.inner {
            #[call(as_os_str)]
            #[expr($.to_str().unwrap())]
            fn as_str(self) -> &'a str;
        }
    }
}

impl AsRef<str> for Component<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    inner: std::path::Iter<'a>
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| s.to_str().explicit_unwrap())
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|s| s.to_str().explicit_unwrap())
    }
}

impl FusedIterator for Iter<'_> {}

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PathBuf {
    inner: std::path::PathBuf
}

impl PathBuf {
    pub fn new() -> PathBuf {
        PathBuf { inner: std::path::PathBuf::new() }
    }

    pub fn with_capacity(capacity: usize) -> PathBuf {
        PathBuf { inner: std::path::PathBuf::with_capacity(capacity) }
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.inner.push(&path.as_ref().inner);
    }

    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self.inner.set_file_name(file_name.as_ref());
    }

    pub fn set_extension<S: AsRef<str>>(&mut self, file_name: S) -> bool {
        self.inner.set_extension(file_name.as_ref())
    }

    pub fn add_extension<S: AsRef<str>>(&mut self, file_name: S) -> bool {
        self.inner.add_extension(file_name.as_ref())
    }

    delegate! {
        to self.inner {
            #[expr(Path::from_inner($))]
            pub fn as_path(&self) -> &Path;

            #[expr(Path::from_inner_mut($))]
            pub fn leak<'a>(self) -> &'a mut Path;

            pub fn pop(&mut self) -> bool;

            #[call(into_os_string)]
            #[expr($.into_string().unwrap())]
            pub fn into_string(self) -> String;

            #[expr(unsafe {std::mem::transmute($)})]
            pub fn into_boxed_path(self) -> Box<Path>;

            pub fn capacity(&self) -> usize;

            pub fn clear(&mut self);

            pub fn reserve(&mut self, additional: usize);

            pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;

            pub fn reserve_exact(&mut self, additional: usize);

            pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError>;

            pub fn shrink_to_fit(&mut self);

            pub fn shrink_to(&mut self, min_capacity: usize);
        }
    }
}

impl Debug for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Display for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.to_str(), f)
    }
}

impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        Path::from_inner(self.inner.deref())
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        let os_str: &OsStr = self.inner.as_ref();
        os_str.to_str().explicit_unwrap()
    }
}

impl AsRef<Path> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Borrow<Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        Path::from_inner(self.inner.borrow())
    }
}

impl<P: AsRef<Path>> Extend<P> for PathBuf {
    #[inline]
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        let owned: Vec<P> = iter.into_iter().collect();
        self.inner.extend(owned.iter().map(|p| &p.as_ref().inner));
    }
}

impl<'a> From<&'a PathBuf> for Cow<'a, Path> {
    fn from(value: &'a PathBuf) -> Self {
        let cow: Cow<'a, std::path::Path> = (&value.inner).into();

        unsafe { std::mem::transmute(cow) }
    }
}