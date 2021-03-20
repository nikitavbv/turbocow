use std::collections::HashMap;

#[derive(Debug)]
pub struct HuffmanTree {

    root: HuffmanTreeNode,
}

#[derive(Debug)]
pub struct HuffmanTreeNode {
    
    left: Option<Box<HuffmanTreeNodeElement>>,
    right: Option<Box<HuffmanTreeNodeElement>>
}

#[derive(Debug)]
pub enum HuffmanTreeNodeElement {

    Link(HuffmanTreeNode),
    Value(u8)
}

impl HuffmanTree {

    pub fn new() -> Self {
        HuffmanTree {
            root: HuffmanTreeNode::empty()
        }
    }

    pub fn insert_code(&mut self, code_length: u8, code_value: u8) {
        &self.root.insert_code(code_length, code_value);
    }

    pub fn to_map(&self) -> HashMap<u16, u8> {
        self.root.to_map(0)
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

    pub fn to_map(&self, prefix: u16) -> HashMap<u16, u8> {
        let mut map = HashMap::new();

        let left_entries = match &self.left {
            None => HashMap::new(),
            Some(left) => left.to_map(prefix << 1)
        };
        let right_entries = match &self.right {
            None => HashMap::new(),
            Some(right) => right.to_map((prefix << 1) + 1)
        };

        map.extend(left_entries);
        map.extend(right_entries);

        map
    }
}

impl HuffmanTreeNodeElement {
    
    fn to_map(&self, prefix: u16) -> HashMap<u16, u8> {
        match &self {
            HuffmanTreeNodeElement::Value(v) => {
                let mut map = HashMap::new();
                map.insert(prefix, *v);
                map
            },
            HuffmanTreeNodeElement::Link(node) => node.to_map(prefix)
        }
    }
}
