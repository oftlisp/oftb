use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ops::{Index, IndexMut};

use symbol::Symbol;

use util::AsPointer;

/// A heap of OftLisp values.
pub struct Heap {
    cells: Vec<HeapCell>,
}

impl Heap {
    /// Creates a new, empty heap.
    pub fn new() -> Heap {
        Heap {
            cells: vec![HeapCell::Nil],
        }
    }

    /// Allocates the given cell onto the heap, returning its address.
    pub fn alloc_cell(&mut self, cell: HeapCell) -> usize {
        if cell == HeapCell::Nil {
            0
        } else {
            let addr = self.cells.len();
            self.cells.push(cell);
            addr
        }
    }

    /// Allocates an iterator onto the heap as a list, returning the address it
    /// was placed at.
    pub fn alloc_iter_as_list<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = HeapCell>,
        I::IntoIter: DoubleEndedIterator,
    {
        self.alloc_iter_as_list_rev(iter.into_iter().rev())
    }

    /// Allocates an iterator onto the heap as a list in reverse order,
    /// returning the address it was placed at.
    pub fn alloc_iter_as_list_rev<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = HeapCell>,
    {
        let mut addr = 0;
        for cell in iter {
            let val = self.alloc_cell(cell);
            addr = self.alloc_cell(HeapCell::Cons(val, addr));
        }
        addr
    }

    /// Allocates an iterator onto the heap as a vector, returning the address
    /// it was placed at.
    pub fn alloc_iter_as_vector<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = HeapCell>,
    {
        let addrs =
            iter.into_iter().map(|cell| self.alloc_cell(cell)).collect();
        self.alloc_cell(HeapCell::Vector(addrs))
    }

    /// Gets a `HeapRef` to a given place.
    pub fn get(&self, addr: usize) -> HeapRef {
        HeapRef(self, addr)
    }
}

impl Index<usize> for Heap {
    type Output = HeapCell;
    fn index(&self, i: usize) -> &HeapCell {
        &self.cells[i]
    }
}

impl IndexMut<usize> for Heap {
    fn index_mut(&mut self, i: usize) -> &mut HeapCell {
        &mut self.cells[i]
    }
}

/// A single cell in a heap.
#[derive(Debug, PartialEq)]
pub enum HeapCell {
    Byte(u8),
    Bytes(Vec<u8>),
    Cons(usize, usize),
    Fixnum(usize),
    Nil,
    String(String),
    Symbol(Symbol),
    Vector(Vec<usize>),
}

/// A helper struct for most operations involving a heap cell.
#[derive(Clone, Copy)]
pub struct HeapRef<'a>(&'a Heap, usize);

impl<'a> HeapRef<'a> {
    fn cell(&self) -> &'a HeapCell {
        &self.0[self.1]
    }
}

impl<'a> Debug for HeapRef<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("HeapRef")
            .field(&AsPointer(&self.0))
            .field(&self.1)
            .finish()
    }
}

impl<'a> Display for HeapRef<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self.cell() {
            HeapCell::Byte(n) => write!(fmt, "{}", n),
            HeapCell::Bytes(ref bs) => {
                write!(fmt, "b\"")?;
                for b in bs {
                    write!(fmt, "\\x{:02x}", b)?;
                }
                write!(fmt, "\"")
            }
            HeapCell::Cons(h, t) => {
                write!(fmt, "({}", HeapRef(self.0, h))?;
                let mut addr = t;
                loop {
                    match *HeapRef(self.0, addr).cell() {
                        HeapCell::Cons(h, t) => {
                            write!(fmt, " {}", HeapRef(self.0, h))?;
                            addr = t;
                        }
                        HeapCell::Nil => break,
                        _ => {
                            write!(fmt, " | {}", HeapRef(self.0, addr))?;
                            break;
                        }
                    }
                }
                write!(fmt, ")")
            }
            HeapCell::Fixnum(n) => write!(fmt, "{}", n),
            HeapCell::Nil => write!(fmt, "()"),
            HeapCell::String(ref s) => {
                write!(fmt, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '\n' => write!(fmt, "\\n")?,
                        '\r' => write!(fmt, "\\r")?,
                        '\t' => write!(fmt, "\\t")?,
                        '\\' => write!(fmt, "\\\\")?,
                        '\"' => write!(fmt, "\\\"")?,
                        _ => write!(fmt, "{}", ch)?,
                    }
                }
                write!(fmt, "\"")
            }
            HeapCell::Symbol(s) => write!(fmt, "{}", s),
            HeapCell::Vector(ref vs) => {
                write!(fmt, "[")?;
                let mut first = true;
                for &addr in vs {
                    if first {
                        first = false;
                    } else {
                        write!(fmt, " ")?;
                    }
                    write!(fmt, "{}", HeapRef(self.0, addr))?;
                }
                write!(fmt, "]")
            }
        }
    }
}
