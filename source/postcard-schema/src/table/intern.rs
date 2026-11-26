//! Interning names in linear time, for both table builders
//!
//! A name is reused if its bytes already occur anywhere in the names so far,
//! even inside another name or across two of them, and the first occurrence is
//! used. Searching the names for each new name would take
//! quadratic time, so this keeps a [suffix automaton] of the names: a graph
//! whose paths from the root spell every substring of the names, built as bytes
//! are appended. Each state records where its substrings first end, which gives
//! the first occurrence directly.
//!
//! For `len` bytes of names there are at most `2 * len + 1` states and
//! `3 * len + 1` transitions, stored in flat arrays. A state's transitions are
//! a list of at most 256, searched in order.
//!
//! [suffix automaton]: https://en.wikipedia.org/wiki/Suffix_automaton
//!
//! These are `const fn`s over slices, so the const builder uses them too, with
//! space in its scratch. Both builders choose the same names.

use super::{to_u32, NameRef};

const NONE: u32 = u32::MAX;

/// A state of the automaton
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct State {
    /// The length of the longest substring that reaches this state
    len: u32,
    /// The state of the longest suffix of this state's substrings that belongs
    /// to another state, or `NONE`
    link: u32,
    /// Where this state's substrings first end: the index of their last byte in
    /// the names
    first: u32,
    /// The first of this state's transitions, or `NONE`
    head: u32,
}

/// A transition, in a list of a state's transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Edge {
    byte: u8,
    to: u32,
    next: u32,
}

pub(crate) const EMPTY_STATE: State = State {
    len: 0,
    link: NONE,
    first: 0,
    head: NONE,
};
pub(crate) const EMPTY_EDGE: Edge = Edge {
    byte: 0,
    to: NONE,
    next: NONE,
};

/// The number of states needed to intern `name_bytes` bytes of names
pub const fn states_for(name_bytes: usize) -> usize {
    2 * name_bytes + 1
}

/// The number of transitions needed to intern `name_bytes` bytes of
/// names
pub const fn edges_for(name_bytes: usize) -> usize {
    3 * name_bytes + 1
}

/// The names, and the automaton of them
pub(crate) struct Interner<'a> {
    pub(crate) names: &'a mut [u8],
    pub(crate) n_names: usize,
    states: &'a mut [State],
    n_states: usize,
    edges: &'a mut [Edge],
    n_edges: usize,
    /// The state of the whole of the names so far
    last: u32,
}

impl<'a> Interner<'a> {
    /// An empty interner, using the given space. The space for states and edges
    /// must be at least [`states_for`] and [`edges_for`] the space for names.
    pub(crate) const fn new(
        names: &'a mut [u8],
        states: &'a mut [State],
        edges: &'a mut [Edge],
    ) -> Self {
        assert!(
            states.len() >= states_for(names.len()),
            "out of state space"
        );
        assert!(edges.len() >= edges_for(names.len()), "out of edge space");
        states[0] = EMPTY_STATE;
        Self {
            names,
            n_names: 0,
            states,
            n_states: 1,
            edges,
            n_edges: 0,
            last: 0,
        }
    }

    /// Returns a reference to `name`, adding it if it doesn't already occur in
    /// the names.
    pub(crate) const fn intern(&mut self, name: &str) -> NameRef {
        let bytes = name.as_bytes();
        if let Some(offset) = self.find(bytes) {
            return NameRef {
                offset,
                len: to_u32(bytes.len()),
            };
        }
        assert!(
            self.n_names + bytes.len() <= self.names.len(),
            "out of name space"
        );
        let offset = self.n_names;
        let mut i = 0;
        while i < bytes.len() {
            self.names[self.n_names] = bytes[i];
            self.extend(bytes[i], self.n_names);
            self.n_names += 1;
            i += 1;
        }
        NameRef {
            offset: to_u32(offset),
            len: to_u32(bytes.len()),
        }
    }

    /// The offset of the first occurrence of `bytes` in the names
    const fn find(&self, bytes: &[u8]) -> Option<u32> {
        if bytes.is_empty() {
            return Some(0);
        }
        let mut state = 0;
        let mut i = 0;
        while i < bytes.len() {
            state = self.transition(state, bytes[i]);
            if state == NONE {
                return None;
            }
            i += 1;
        }
        // `first` is where the occurrence ends, so it starts this far back
        Some(self.states[state as usize].first + 1 - to_u32(bytes.len()))
    }

    const fn transition(&self, state: u32, byte: u8) -> u32 {
        let mut e = self.states[state as usize].head;
        while e != NONE {
            let edge = self.edges[e as usize];
            if edge.byte == byte {
                return edge.to;
            }
            e = edge.next;
        }
        NONE
    }

    const fn add_edge(&mut self, state: u32, byte: u8, to: u32) {
        assert!(self.n_edges < self.edges.len(), "out of edge space");
        self.edges[self.n_edges] = Edge {
            byte,
            to,
            next: self.states[state as usize].head,
        };
        self.states[state as usize].head = to_u32(self.n_edges);
        self.n_edges += 1;
    }

    /// Points `state`'s transition on `byte`, which exists, at `to`
    const fn redirect(&mut self, state: u32, byte: u8, to: u32) {
        let mut e = self.states[state as usize].head;
        while e != NONE {
            if self.edges[e as usize].byte == byte {
                self.edges[e as usize].to = to;
                return;
            }
            e = self.edges[e as usize].next;
        }
    }

    const fn new_state(&mut self, state: State) -> u32 {
        assert!(self.n_states < self.states.len(), "out of state space");
        self.states[self.n_states] = state;
        self.n_states += 1;
        to_u32(self.n_states - 1)
    }

    /// Adds `byte`, at index `pos` of the names, to the automaton. This is
    /// the usual online construction of a suffix automaton.
    const fn extend(&mut self, byte: u8, pos: usize) {
        let cur = self.new_state(State {
            len: self.states[self.last as usize].len + 1,
            link: NONE,
            first: to_u32(pos),
            head: NONE,
        });
        let mut p = self.last;
        while p != NONE && self.transition(p, byte) == NONE {
            self.add_edge(p, byte, cur);
            p = self.states[p as usize].link;
        }
        if p == NONE {
            self.states[cur as usize].link = 0;
        } else {
            let q = self.transition(p, byte);
            if self.states[p as usize].len + 1 == self.states[q as usize].len {
                self.states[cur as usize].link = q;
            } else {
                // Split `q`: the clone takes the shorter substrings, copies
                // `q`'s transitions and keeps its first occurrence.
                let clone = self.new_state(State {
                    len: self.states[p as usize].len + 1,
                    link: self.states[q as usize].link,
                    first: self.states[q as usize].first,
                    head: NONE,
                });
                let mut e = self.states[q as usize].head;
                while e != NONE {
                    let edge = self.edges[e as usize];
                    self.add_edge(clone, edge.byte, edge.to);
                    e = edge.next;
                }
                while p != NONE && self.transition(p, byte) == q {
                    self.redirect(p, byte, clone);
                    p = self.states[p as usize].link;
                }
                self.states[q as usize].link = clone;
                self.states[cur as usize].link = clone;
            }
        }
        self.last = cur;
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    /// The quadratic search that the interner replaces
    fn naive(names: &mut String, name: &str) -> NameRef {
        let offset = match names.find(name) {
            Some(offset) => offset,
            None => {
                names.push_str(name);
                names.len() - name.len()
            }
        };
        NameRef {
            offset: offset as u32,
            len: name.len() as u32,
        }
    }

    /// The interner gives the same names as searching, including matches
    /// inside a name, across two names, and of multi-byte characters.
    #[test]
    fn matches_naive_search() {
        let mut inputs: Vec<String> = [
            "", "xy", "y", "yx", "xyx", "Point", "int", "é", "aé", "éa", "",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        // Short names from a small alphabet, so many are reused
        let alphabet = ['a', 'b', 'é', 'c'];
        let mut seed = 0x1234_5678u32;
        for _ in 0..2000 {
            let mut s = String::new();
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            for k in 0..(seed >> 28) as usize % 6 {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
                s.push(alphabet[((seed >> 16) as usize + k) % alphabet.len()]);
            }
            inputs.push(s);
        }
        let total: usize = inputs.iter().map(|s| s.len()).sum();
        let mut names = vec![0; total];
        let mut states = vec![EMPTY_STATE; states_for(total)];
        let mut edges = vec![EMPTY_EDGE; edges_for(total)];
        let mut interner = Interner::new(&mut names, &mut states, &mut edges);
        let mut expected = String::new();
        for s in &inputs {
            assert_eq!(interner.intern(s), naive(&mut expected, s), "{s:?}");
        }
        let n = interner.n_names;
        assert_eq!(&names[..n], expected.as_bytes());
    }
}
