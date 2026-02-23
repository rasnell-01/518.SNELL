#![allow(unsafe_code)]

use std::fmt;
use std::ptr;

struct Node<T> {
    value: T,
    both: usize,
}

impl<T> Node<T> {
    fn allocate(value: T) -> *mut Self {
        Box::into_raw(Box::new(Node { value, both: 0 }))
    }
}

#[inline]
fn xor<T>(a: *mut Node<T>, b: *mut Node<T>) -> usize {
    (a as usize) ^ (b as usize)
}

#[inline]
fn as_ptr<T>(address: usize) -> *mut Node<T> {
    address as *mut Node<T>
}

pub struct XorList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
}

unsafe impl<T:Send> Send for XorList<T> {}
unsafe impl<T:Sync> Sync for XorList<T> {}

impl<T> XorList<T> {
    pub fn new() -> Self {
        XorList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_front(&mut self, value: T) {
        unsafe {
            let new_node = Node::allocate(value);
            if self.head.is_null() {
                self.head = new_node;
                self.tail = new_node;
            } else {
                (*new_node).both = self.head as usize;
                (*self.head).both ^= new_node as usize;
                self.head = new_node;
            }
            self.len += 1;
        }
    }

    pub fn push_back(&mut self, value: T) {
        unsafe {
            let new_node = Node::allocate(value);
            if self.tail.is_null() {
                self.head = new_node;
                self.tail = new_node;
            } else {
                (*new_node).both = self.tail as usize;
                (*self.tail).both ^= new_node as usize;
                self.tail = new_node;
            }
        }
        self.len += 1;
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if index == 0 {
            return self.push_front(value);
        }
        if index >= self.len {
            return self.push_back(value);
        }
        unsafe {
            let (previous_ptr, current_ptr) = self.walk_to(index);
            let new_node = Node::allocate(value);
            (*new_node).both = xor(previous_ptr, current_ptr);
            (*previous_ptr).both ^= (current_ptr as usize) ^ (new_node as usize);
            (*current_ptr).both ^= (previous_ptr as usize) ^ (new_node as usize);
            self.len += 1;
        }
    }

    pub fn delete(&mut self, index: usize) -> Option<T> {
        if self.is_empty() || index >= self.len {
            return None;
        }
        unsafe {
            let (previous_ptr, current_ptr) = self.walk_to(index);
            let next_ptr: *mut Node<T> = as_ptr((*current_ptr).both ^ previous_ptr as usize);
            if previous_ptr.is_null() {
                self.head = next_ptr;
            } else {
                (*previous_ptr).both ^= (current_ptr as usize) ^ (next_ptr as usize);
            }
            if next_ptr.is_null() {
                self.tail = previous_ptr;
            } else {
                (*next_ptr).both ^= (current_ptr as usize) ^ (previous_ptr as usize);
            }
            self.len -= 1;
            Some(Box::from_raw(current_ptr).value)
        }
    }

    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        unsafe {
            let mut previous: *mut Node<T> = ptr::null_mut();
            let mut current = self.head;
            while !current.is_null(){
                f(&(*current).value);
                let next: *mut Node<T> = as_ptr((*current).both ^ previous as usize);
                previous = current;
                current = next;
            }
        }
    }

    pub fn traverse_reverse<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        unsafe {
            let mut next: *mut Node<T> = ptr::null_mut();
            let mut current = self.tail;
            while !current.is_null(){
                f(&(*current).value);
                let previous: *mut Node<T> = as_ptr((*current).both ^ next as usize);
                next = current;
                current = previous;
            }
        }
    }

    unsafe fn walk_to(&self, index: usize) -> (*mut Node<T>, *mut Node<T>) {
        let mut previous: *mut Node<T> = ptr::null_mut();
        let mut current: *mut Node<T> = self.head;
        for _ in 0..index {
            let next: *mut Node<T> = as_ptr((*current).both ^ previous as usize);
            previous = current;
            current = next;
        }
        (previous, current)
    }
}

impl<T> Default for XorList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for XorList<T> {
    fn drop(&mut self) {
        unsafe {
            let mut previous: *mut Node<T> = ptr::null_mut();
            let mut current = self.head;
            while !current.is_null() {
                let next: *mut Node<T> = as_ptr((*current).both ^ previous as usize);
                previous = current;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for XorList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items: Vec<String> = Vec::with_capacity(self.len);
        self.traverse(|v| items.push(format!("{v:?}")));
        write!(f, "[")?;
        for (i, s) in items.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{s}")?;
        }
        write!(f, "]")
    }
}

pub(crate) fn new() -> XorList<i32> {
    todo!()
}