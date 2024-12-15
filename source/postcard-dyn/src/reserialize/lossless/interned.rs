use core::hash::{Hash, Hasher};

use hashbrown::HashSet;

#[derive(Default)]
pub struct Interned {
    strings: HashSet<&'static str>,
    slices: HashSet<Slice>,
}

impl Interned {
    pub fn intern_identifier(&mut self, s: &str) -> &'static str {
        Self::intern_str(&mut self.strings, s)
    }

    fn intern_str(strings: &mut HashSet<&'static str>, s: &str) -> &'static str {
        strings.get_or_insert_with(s, |s| String::leak(s.to_string()))
    }

    pub fn intern_slice<'a>(
        &mut self,
        strings: impl IntoIterator<Item = &'a str, IntoIter: Clone>,
    ) -> &'static [&'static str] {
        let Slice(slice) = self
            .slices
            .get_or_insert_with(&Iter(strings.into_iter()), |elements| {
                let strings = elements.0.clone();
                let interned = strings.map(|s| Self::intern_str(&mut self.strings, s));
                Slice(Box::leak(interned.collect()))
            });
        slice
    }
}

#[derive(PartialEq, Eq)]
struct Slice(&'static [&'static str]);
struct Iter<'a, I: Iterator<Item = &'a str>>(I);

impl Hash for Slice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for s in self.0 {
            s.hash(state)
        }
    }
}

impl<'a, I: Iterator<Item = &'a str> + Clone> Hash for Iter<'a, I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for s in self.0.clone() {
            s.hash(state)
        }
    }
}

impl<'a, I> hashbrown::Equivalent<Slice> for Iter<'a, I>
where
    I: Iterator<Item = &'a str> + Clone,
{
    fn equivalent(&self, slice: &Slice) -> bool {
        self.0.clone().eq(slice.0.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::Interned;

    #[test]
    fn basic() {
        let mut interned = Interned::default();

        assert_eq!(interned.intern_identifier("hello"), "hello");

        let slices: &[&[&str]] = &[&[], &["foo"], &["foo", "bar"]];
        for &slice in slices {
            assert_eq!(interned.intern_slice(slice.iter().copied()), slice);
        }
        assert_eq!(interned.slices.len(), slices.len());
    }
}
