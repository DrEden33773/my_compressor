#[allow(dead_code)]
struct Info {
  n_bit: usize,
  n_box: usize,
  curr_i_box: usize,
}

impl Default for Info {
  fn default() -> Self {
    Self {
      n_bit: 0,
      n_box: 1,
      curr_i_box: 0,
    }
  }
}

const BOX_SIZE: usize = std::mem::size_of::<usize>();

pub struct DynamicBitset {
  data: Vec<usize>,
  n_bit: usize,
  n_box: usize,
  curr_i_box: usize,
}

impl From<String> for DynamicBitset {
  fn from(value: String) -> Self {
    let mut res = Self::default();
    for bit in value.chars() {
      res.push(bit == '1');
    }
    res
  }
}

impl From<DynamicBitset> for String {
  fn from(val: DynamicBitset) -> Self {
    let mut res = String::new();
    val.for_each_bit(|bit| {
      res += if bit != 1 { "1" } else { "0" };
    });
    res
  }
}

impl DynamicBitset {
  pub fn push(&mut self, value: bool) {
    self.check_if_add_box();
    if value {
      // `00000000 1 00000000` like
      self.data[self.curr_i_box] |= 1 << (self.n_bit % BOX_SIZE);
    } else {
      // `11111111 0 11111111` like
      self.data[self.curr_i_box] &= !(1 << (self.n_bit % BOX_SIZE));
    }
    self.n_bit += 1;
  }

  pub fn pop(&mut self) {
    /*
      This function should make sure:
        1. Each unused bit is set to `0`
        2. Each popped bit will be reset to `0`
    */
    self.data[self.curr_i_box] &= !(1 << (self.n_bit % BOX_SIZE));
    self.n_bit -= 1;
    self.check_if_sub_box();
  }

  pub fn back(&self) -> bool {
    self.data[self.curr_i_box] & (1 << (self.n_bit % BOX_SIZE)) != 0
  }
}

impl DynamicBitset {
  pub fn for_each_bit(&self, mut func: impl FnMut(usize)) {
    // 1. box <- [0..n_box - 1]
    for i in 0..self.n_box - 1 {
      let curr_box = self.data[i];
      for j in 0..BOX_SIZE {
        func(curr_box & (1 << j));
      }
    }
    // 2. box <- [n_box - 1]
    let last_box = self.data[self.n_box - 1];
    for j in 0..self.n_bit % BOX_SIZE {
      func(last_box & (1 << j));
    }
  }

  pub fn for_each_box(&self, mut func: impl FnMut(usize)) {
    for i in 0..self.n_box {
      func(self.data[i]);
    }
  }
}

impl DynamicBitset {
  fn check_if_add_box(&mut self) {
    let if_add_box = self.n_bit % BOX_SIZE == 0 && self.n_bit > 0;
    if !if_add_box {
      return;
    }
    self.data.push(0);
    self.n_box += 1;
    self.curr_i_box += 1;
  }

  fn check_if_sub_box(&mut self) {
    let if_sub_box = self.n_bit % BOX_SIZE == 0 && self.n_bit > 1;
    if !if_sub_box {
      return;
    }
    self.data.pop();
    self.n_box -= 1;
    self.curr_i_box -= 1;
  }
}

impl Default for DynamicBitset {
  fn default() -> Self {
    Self {
      data: vec![usize::MAX],
      n_bit: 0,
      n_box: 1,
      curr_i_box: 0,
    }
  }
}
