use std::fmt;
use std::option::Option;
use std::fmt::Display;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node { value, next }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList { head: None, size: 0 }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }

    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl<T> fmt::Display for LinkedList<T>
    where T: Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                }
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T> Clone for Node<T>
    where T: Clone {
    fn clone(&self) -> Self {
        Node::new(self.value.clone(), self.next.clone())
    }
}

impl<T> Clone for LinkedList<T>
    where T: Clone {
    fn clone(&self) -> Self {
        let mut list = LinkedList::new();
        list.head = self.head.clone();
        list.size = self.size;
        list
    }
}


impl<T> PartialEq for Node<T>
    where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> PartialEq for LinkedList<T>
    where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }
        if self.size != 0 {
            let mut self_node = &self.head;
            let mut other_node = &other.head;
            while self_node.is_some() && other_node.is_some() {
                if self_node.as_ref().unwrap() != other_node.as_ref().unwrap() {
                    return false;
                }
                self_node = &self_node.as_ref().unwrap().next;
                other_node = &other_node.as_ref().unwrap().next;
            }
            if self_node.is_some() || other_node.is_some() {
                return false;
            }
        }
        true
    }
}

pub struct LinkedListIter1<T> {
    current: Option<Box<Node<T>>>,
}

impl<T> Iterator for LinkedListIter1<T>
    where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.current {
            Some(node) => {
                let val = node.value.clone();
                self.current = node.next.clone();
                Some(val)
            }
            None => None
        }
    }
}

impl<T> IntoIterator for LinkedList<T>
    where T: Clone {
    type Item = T;
    type IntoIter = LinkedListIter1<T>;
    fn into_iter(self) -> LinkedListIter1<T> {
        LinkedListIter1 { current: self.head.clone() }
    }
}

pub struct LinkedListIter<'a, T> {
    current: &'a Option<Box<Node<T>>>,
}


impl<T> Iterator for LinkedListIter<'_, T>
where T:Clone{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.current {
            Some(node) => {
                self.current = &node.next;
                Some(node.value.clone())
            }
            None => None
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T>
    where T: Clone {
    type Item = T;
    type IntoIter = LinkedListIter<'a, T>;
    fn into_iter(self) -> LinkedListIter<'a, T> {
        LinkedListIter { current: &self.head }
    }
}