use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};

#[derive(Debug)]
pub struct HuffmanTree {

    root: HuffmanTreeNode,
}

#[derive(Debug)]
pub struct HuffmanTreeNode {
    
    left: Option<Box<HuffmanTreeNodeElement>>,
    right: Option<Box<HuffmanTreeNodeElement>>
}

impl PartialEq for HuffmanTreeNode {

    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl Eq for HuffmanTreeNode {
}

impl HuffmanTreeNode {

    fn with_two_subnodes(left: HuffmanTreeNodeElement, right: HuffmanTreeNodeElement) -> Self {
        Self {
            left: Some(box left),
            right: Some(box right),
        }
    }
}

#[derive(Debug)]
pub enum HuffmanTreeNodeElement {

    Link(HuffmanTreeNode),
    Value(u8)
}

impl HuffmanTreeNodeElement {

    fn is_link(&self) -> bool {
        match self {
            Self::Link(_) => true,
            _ => false,
        }
    }

    fn is_value(&self) -> bool {
        match self {
            Self::Value(_) => true,
            _ => false,
        }
    }
}

impl PartialEq for HuffmanTreeNodeElement {
    
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Value(v1) => match other {
                Self::Link(_) => false,
                Self::Value(v2) => v1 == v2,
            },
            Self::Link(link1) => match other {
                Self::Value(_) => false,
                Self::Link(link2) => link1 == link2,
            },
        }
    }
}

impl Eq for HuffmanTreeNodeElement {
}

pub struct HuffmanTreeBuilder {

    values: HashMap<u8, usize>,
}

#[derive(Debug)]
pub struct HuffmanTreeBuilderEntry {

    frequency: usize,
    node: HuffmanTreeNodeElement,
}

impl PartialEq for HuffmanTreeBuilderEntry {

    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency && self.node == other.node
    }
}

impl Eq for HuffmanTreeBuilderEntry {
}

impl Ord for HuffmanTreeBuilderEntry {

    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for HuffmanTreeBuilderEntry {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.frequency == other.frequency {
            return if self.node.is_link() && other.node.is_value() {
                Some(Ordering::Less)
            } else if self.node.is_value() && other.node.is_link() {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }

        Some(other.frequency.cmp(&self.frequency))
    }
}

impl HuffmanTreeBuilderEntry {

    fn new(frequency: usize, node: HuffmanTreeNodeElement) -> Self {
        Self {
            frequency,
            node,
        }
    }
}

impl HuffmanTree {

    pub fn new() -> Self {
        Self::with_root(HuffmanTreeNode::empty())
    }

    pub fn with_root(root: HuffmanTreeNode) -> Self {
        HuffmanTree {
            root,
        }
    }

    pub fn insert_code(&mut self, code_length: u8, code_value: u8) {
        &self.root.insert_code(code_length, code_value);
    }

    pub fn to_map(&self) -> HashMap<(u16, u16), u8> {
        self.root.to_map(0, 0)
    }
}

impl HuffmanTreeNode {

    fn empty() -> Self {
        HuffmanTreeNode {
            left: None,
            right: None,
        }
    }

    fn insert_code(&mut self, code_length: u8, code_value: u8) -> bool {
        //panic!("don't know what to do with: {} {}", code_length, code_value);

        let left = &mut self.left;
        let right = &mut self.right;

        if code_length == 0 {
            if left.is_none() {
                self.left = Some(box HuffmanTreeNodeElement::Value(code_value));
                true
            } else if right.is_none() {
                self.right = Some(box HuffmanTreeNodeElement::Value(code_value));
                true
            } else {
                false
            }
        } else {
            if left.is_some() {
                let left = left.as_mut()
                    .expect("left should be present here, because we checked for this previously")
                    .as_mut();
            
                if let HuffmanTreeNodeElement::Link(left_node) = left {
                    // try inserting into left subtree
                    if left_node.insert_code(code_length - 1, code_value) {
                        // success
                        true
                    } else {
                        // let's try right subtree then
                        if right.is_some() {
                            let right = right.as_mut()
                                .expect("right should be present here, because we checked for this previously")
                                .as_mut();

                            if let HuffmanTreeNodeElement::Link(right_node) = right {
                                right_node.insert_code(code_length - 1, code_value)
                            } else {
                                // right node contains value, we can't insert here, let's return false and try somewhere else
                                false
                            }
                        } else {
                            // right node is empty, let's create a new one!
                            let mut new_node = HuffmanTreeNode::empty();
                            new_node.insert_code(code_length - 1, code_value);
                            self.right = Some(box HuffmanTreeNodeElement::Link(new_node));
                            true
                        }
                    }
                } else {
                    // left node contains value, let's try right one then
                    if right.is_some() {
                        let right = right.as_mut()
                            .expect("right should be present here, because we checked for this previously")
                            .as_mut();

                        if let HuffmanTreeNodeElement::Link(right_node) = right {
                            right_node.insert_code(code_length - 1, code_value)
                        } else {
                            // right node contains value, we can't insert here, let's return false and try somewhere else
                            false
                        }
                    } else {
                        // right node is empty, let's create a new one!
                        let mut new_node = HuffmanTreeNode::empty();
                        new_node.insert_code(code_length - 1, code_value);
                        self.right = Some(box HuffmanTreeNodeElement::Link(new_node));
                        true
                    }
                }
            } else {
                // left node is empty, let's create a new one!
                let mut new_node = HuffmanTreeNode::empty();
                new_node.insert_code(code_length - 1, code_value);
                self.left = Some(box HuffmanTreeNodeElement::Link(new_node));
                true
            }
        }
    }

    pub fn to_map(&self, prefix: u16, length: u16) -> HashMap<(u16, u16), u8> {
        let mut map = HashMap::new();

        let left_entries = match &self.left {
            None => HashMap::new(),
            Some(left) => left.to_map(prefix << 1, length + 1)
        };
        let right_entries = match &self.right {
            None => HashMap::new(),
            Some(right) => right.to_map((prefix << 1) + 1, length + 1)
        };

        map.extend(left_entries);
        map.extend(right_entries);

        map
    }
}

impl HuffmanTreeNodeElement {
    
    fn to_map(&self, prefix: u16, length: u16) -> HashMap<(u16, u16), u8> {
        match &self {
            HuffmanTreeNodeElement::Value(v) => {
                let mut map = HashMap::new();
                map.insert((prefix, length - 1), *v);
                map
            },
            HuffmanTreeNodeElement::Link(node) => node.to_map(prefix, length)
        }
    }
}

impl HuffmanTreeBuilder {

    pub fn new() -> Self {
        HuffmanTreeBuilder {
            values: HashMap::new(),
        }
    }

    pub fn append(&mut self, value: u8) {
        self.append_count(value, 1)
    } 

    pub fn append_count(&mut self, value: u8, inc: usize) {
        let counter = match self.values.get(&value) {
            Some(v) => *v,
            None => 0
        } + inc;

        self.values.insert(value, counter);
    }

    pub fn build(&self) -> HuffmanTree {
        let mut heap: BinaryHeap<HuffmanTreeBuilderEntry> = BinaryHeap::new();

        for (entry, frequency) in &self.values {
            heap.push(HuffmanTreeBuilderEntry::new(*frequency, HuffmanTreeNodeElement::Value(*entry)));
        }

        while heap.len() > 1 {
            let lowest = heap.pop().unwrap();
            let second_lowest = heap.pop().unwrap();
            
            heap.push(HuffmanTreeBuilderEntry::new(
                lowest.frequency + second_lowest.frequency,
                HuffmanTreeNodeElement::Link(HuffmanTreeNode::with_two_subnodes(
                    lowest.node,
                    second_lowest.node,
                ))
            ));
        }

        HuffmanTree::with_root(match heap.pop().unwrap().node {
            HuffmanTreeNodeElement::Link(linked_node) => linked_node,
            HuffmanTreeNodeElement::Value(_) => panic!("this cannot be value"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let mut builder = HuffmanTreeBuilder::new();
        builder.append_count(b'a', 15);
        builder.append_count(b'b', 8);
        builder.append_count(b'c', 7);
        builder.append_count(b'd', 6);
        builder.append_count(b'e', 5);

        let tree = builder.build().to_map();
        assert_eq!(tree.len(), 5);
        assert_eq!(*tree.get(&(5, 2)).unwrap(), 100);
    }
}