use std::{
  collections::{HashMap, VecDeque},
  fmt::{Debug, Display},
  hash::Hash,
};

#[derive(Debug, Clone)]
struct Node<T: Ord + Display + Clone> {
  value: Option<T>,
  index: usize,
  weight: usize,
  bits_string: String,
  i_parent: Option<usize>,
  i_left: Option<usize>,
  i_right: Option<usize>,
}

impl<T: Ord + Display + Clone> Default for Node<T> {
  fn default() -> Self {
    Self {
      value: None,
      index: 0,
      weight: 0,
      bits_string: String::new(),
      i_parent: None,
      i_left: None,
      i_right: None,
    }
  }
}

impl<T: Ord + Display + Clone> PartialEq for Node<T> {
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}

impl<T: Ord + Display + Clone> Display for Node<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}{:12}{:12}{:12?}{:12?}{:12?}",
      self.index,
      self.value.as_ref().unwrap(),
      self.weight,
      self.i_parent,
      self.i_left,
      self.i_right
    )?;
    Ok(())
  }
}

pub struct HuffmanTree<T: Ord + Display + Hash + Clone> {
  nodes: Vec<Node<T>>,
  bits_map: HashMap<T, String>,
  sizeof_table: usize,
  n_init: usize,
  n_input: usize,
}

impl<T: Ord + Display + Hash + Clone> Default for HuffmanTree<T> {
  fn default() -> Self {
    Self {
      nodes: Default::default(),
      bits_map: Default::default(),
      sizeof_table: Default::default(),
      n_init: Default::default(),
      n_input: Default::default(),
    }
  }
}

impl<T: Ord + Display + Hash + Clone> HuffmanTree<T> {
  pub fn new(init_list: &mut Vec<(T, usize)>) -> Self {
    let mut res = Self::default();
    res.generate(init_list);
    res
  }
}

impl<T: Ord + Display + Hash + Clone> HuffmanTree<T> {
  fn sort_then_unique(init_list: &mut Vec<(T, usize)>) {
    init_list.sort();
    init_list.dedup();
  }

  fn alloc(&mut self, init_list: &[(T, usize)]) {
    self.sizeof_table = 2 * init_list.len() - 1;
    self.nodes = vec![Node::default(); self.sizeof_table];
  }

  fn pre_build(&mut self, init_list: &[(T, usize)]) {
    let mut i = 0;
    for (value, weight) in init_list {
      self.nodes[i].index = i;
      self.nodes[i].value = Some(value.clone());
      self.nodes[i].weight = *weight;
      i += 1;
    }
    while i < self.sizeof_table {
      self.nodes[i].index = i;
      i += 1;
    }
  }

  fn find_min2(&self, right: usize) -> (usize, usize) {
    let mut i_min = 0;
    let mut i_second_min = 0;
    let mut min_weight = usize::MAX;
    let mut second_min_weight = usize::MAX;
    for i in 0..right {
      if self.nodes[i].weight < min_weight {
        second_min_weight = min_weight;
        i_second_min = i_min;
        min_weight = self.nodes[i].weight;
        i_min = i;
      } else if self.nodes[i].weight < second_min_weight {
        second_min_weight = self.nodes[i].weight;
        i_second_min = i;
      }
    }
    (i_min, i_second_min)
  }

  fn build(&mut self) {
    let mut i = self.n_init;
    while i < self.sizeof_table {
      let (min, second_min) = self.find_min2(i);
      self.nodes[i].weight = self.nodes[min].weight + self.nodes[second_min].weight;
      self.nodes[i].i_left = Some(min);
      self.nodes[i].i_right = Some(second_min);
      self.nodes[min].i_parent = Some(i);
      self.nodes[second_min].i_parent = Some(i);
      i += 1;
    }
  }

  fn bits_gen(&mut self) {
    let mut queue = VecDeque::new();
    queue.push_back(self.nodes.last().unwrap().index);
    while !queue.is_empty() {
      let i_curr = *queue.front().unwrap();
      let i_left = self.nodes[i_curr].i_left;
      let i_right = self.nodes[i_curr].i_right;
      if let Some(i_left) = i_left {
        self.nodes[i_left].bits_string = self.nodes[i_curr].bits_string.clone() + "0";
        queue.push_back(i_left);
      }
      if let Some(i_right) = i_right {
        self.nodes[i_right].bits_string = self.nodes[i_curr].bits_string.clone() + "1";
        queue.push_back(i_right);
      }
      queue.pop_front();
    }
  }

  fn bits_map_gen(&mut self) {
    for node in &self.nodes[..self.n_input] {
      self
        .bits_map
        .insert(node.value.to_owned().unwrap(), node.bits_string.to_owned());
    }
  }

  fn generate(&mut self, init_list: &mut Vec<(T, usize)>) {
    if init_list.is_empty() {
      return;
    }
    Self::sort_then_unique(init_list);
    self.n_init = init_list.len();
    self.n_input = init_list.len();
    self.alloc(init_list);
    self.pre_build(init_list);
    self.build();
    self.bits_gen();
    self.bits_map_gen();
  }
}
