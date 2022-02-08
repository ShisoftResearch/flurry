use crate::iter::*;
use crate::reclaim::{Guard, GuardRef};
use crate::{HashMap, TryInsertError};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};
use std::ops::Index;

/// A reference to a [`HashMap`], constructed with [`HashMap::pin`] or [`HashMap::with_guard`].
///
/// The current thread will be pinned for the duration of this reference.
/// Keep in mind that this prevents the collection of garbage generated by the map.
pub struct HashMapRef<'map, K, V, S = crate::DefaultHashBuilder> {
    pub(crate) map: &'map HashMap<K, V, S>,
    guard: GuardRef<'map>,
}

impl<K, V, S> HashMap<K, V, S> {
    /// Get a reference to this map with the current thread pinned.
    ///
    /// Keep in mind that for as long as you hold onto this, you are preventing the collection of
    /// garbage generated by the map.
    pub fn pin(&self) -> HashMapRef<'_, K, V, S> {
        HashMapRef {
            guard: GuardRef::Owned(self.guard()),
            map: self,
        }
    }

    /// Get a reference to this map with the given guard.
    pub fn with_guard<'g>(&'g self, guard: &'g Guard<'_>) -> HashMapRef<'g, K, V, S> {
        HashMapRef {
            guard: GuardRef::Ref(guard),
            map: self,
        }
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S> {
    /// Returns the number of entries in the map.
    ///
    /// See also [`HashMap::len`].
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map is empty. Otherwise returns `false`.
    ///
    /// See also [`HashMap::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    ///
    /// The iterator element type is `(&'g K, &'g V)`.
    ///
    /// See also [`HashMap::iter`].
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.map.iter(&self.guard)
    }

    /// An iterator visiting all keys in arbitrary order.
    ///
    /// The iterator element type is `&'g K`.
    ///
    /// See also [`HashMap::keys`].
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.map.keys(&self.guard)
    }

    /// An iterator visiting all values in arbitrary order.
    ///
    /// The iterator element type is `&'g V`.
    ///
    /// See also [`HashMap::values`].
    pub fn values(&self) -> Values<'_, K, V> {
        self.map.values(&self.guard)
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S>
where
    K: Clone + Ord,
{
    /// Tries to reserve capacity for at least `additional` more elements to be inserted in the
    /// `HashMap`.
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// See also [`HashMap::reserve`].
    pub fn reserve(&self, additional: usize) {
        self.map.reserve(additional, &self.guard)
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S>
where
    K: Hash + Ord,
    S: BuildHasher,
{
    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// See also [`HashMap::contains_key`].
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.contains_key(key, &self.guard)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// See also [`HashMap::get`].
    #[inline]
    pub fn get<'g, Q>(&'g self, key: &Q) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.get(key, &self.guard)
    }

    /// Returns the key-value pair corresponding to `key`.
    ///
    /// See also [`HashMap::get_key_value`].
    #[inline]
    pub fn get_key_value<'g, Q>(&'g self, key: &Q) -> Option<(&'g K, &'g V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.get_key_value(key, &self.guard)
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S>
where
    K: Clone + Ord,
{
    /// Clears the map, removing all key-value pairs.
    ///
    /// See also [`HashMap::clear`].
    pub fn clear(&self) {
        self.map.clear(&self.guard);
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S>
where
    K: 'static + Sync + Send + Clone + Hash + Ord,
    V: 'static + Sync + Send,
    S: BuildHasher,
{
    /// Inserts a key-value pair into the map.
    ///
    /// See also [`HashMap::insert`].
    pub fn insert(&self, key: K, value: V) -> Option<&'_ V> {
        self.map.insert(key, value, &self.guard)
    }

    /// Inserts a key-value pair into the map unless the key already exists.
    ///
    /// See also [`HashMap::try_insert`].
    #[inline]
    pub fn try_insert(&self, key: K, value: V) -> Result<&'_ V, TryInsertError<'_, V>> {
        self.map.try_insert(key, value, &self.guard)
    }

    /// If the value for the specified `key` is present, attempts to
    /// compute a new mapping given the key and its current mapped value.
    ///
    /// See also [`HashMap::compute_if_present`].
    pub fn compute_if_present<'g, Q, F>(&'g self, key: &Q, remapping_function: F) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
        F: FnOnce(&K, &V) -> Option<V>,
    {
        self.map
            .compute_if_present(key, remapping_function, &self.guard)
    }

    /// Removes a key-value pair from the map, and returns the removed value (if any).
    ///
    /// See also [`HashMap::remove`].
    pub fn remove<'g, Q>(&'g self, key: &Q) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.remove(key, &self.guard)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// See also [`HashMap::remove_entry`].
    pub fn remove_entry<'g, Q>(&'g self, key: &Q) -> Option<(&'g K, &'g V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.remove_entry(key, &self.guard)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// See also [`HashMap::retain`].
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        self.map.retain(f, &self.guard);
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// See also [`HashMap::retain_force`].
    pub fn retain_force<F>(&self, f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        self.map.retain_force(f, &self.guard);
    }
}

impl<'g, K, V, S> IntoIterator for &'g HashMapRef<'_, K, V, S> {
    type IntoIter = Iter<'g, K, V>;
    type Item = (&'g K, &'g V);

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter(&self.guard)
    }
}

impl<K, V, S> Debug for HashMapRef<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<K, V, S> Clone for HashMapRef<'_, K, V, S> {
    fn clone(&self) -> Self {
        self.map.pin()
    }
}

impl<K, V, S> PartialEq for HashMapRef<'_, K, V, S>
where
    K: Hash + Ord,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.map.guarded_eq(other.map, &self.guard, &other.guard)
    }
}

impl<K, V, S> PartialEq<HashMap<K, V, S>> for HashMapRef<'_, K, V, S>
where
    K: Hash + Ord,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        self.map.guarded_eq(other, &self.guard, &other.guard())
    }
}

impl<K, V, S> PartialEq<HashMapRef<'_, K, V, S>> for HashMap<K, V, S>
where
    K: Hash + Ord,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMapRef<'_, K, V, S>) -> bool {
        self.guarded_eq(other.map, &self.guard(), &other.guard)
    }
}

impl<K, V, S> Eq for HashMapRef<'_, K, V, S>
where
    K: Hash + Ord,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, Q, V, S> Index<&'_ Q> for HashMapRef<'_, K, V, S>
where
    K: Hash + Ord + Borrow<Q>,
    Q: ?Sized + Hash + Ord,
    S: BuildHasher,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}
