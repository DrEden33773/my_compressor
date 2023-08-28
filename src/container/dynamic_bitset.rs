pub struct Info {
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

#[derive(Debug)]
pub struct DynamicBitset {
  data: Vec<usize>,
  n_bit: usize,
  n_box: usize,
  curr_i_box: usize,
}

impl PartialEq for DynamicBitset {
  fn eq(&self, other: &Self) -> bool {
    /*
      Must guarantee:
        unused/popped bit == 0
    */
    self.data == other.data && self.n_bit == other.n_bit
  }
}
impl Eq for DynamicBitset {}

impl DynamicBitset {
  pub fn sync_with_info(&mut self, info: &Info) {
    self.n_bit = info.n_bit;
    self.n_box = info.n_box;
    self.curr_i_box = info.curr_i_box;
  }

  pub fn get_info(&self) -> Info {
    Info {
      n_bit: self.n_bit,
      n_box: self.n_box,
      curr_i_box: self.curr_i_box,
    }
  }
}

impl From<String> for DynamicBitset {
  fn from(value: String) -> Self {
    let mut res = Self::default();
    for bit in value.chars() {
      res.push(bit != '0');
    }
    res
  }
}
impl From<Vec<bool>> for DynamicBitset {
  fn from(value: Vec<bool>) -> Self {
    let mut res = Self::default();
    for bit in value {
      res.push(bit);
    }
    res
  }
}
impl From<&str> for DynamicBitset {
  fn from(value: &str) -> Self {
    let mut res = Self::default();
    for bit in value.chars() {
      res.push(bit != '0');
    }
    res
  }
}

impl From<&DynamicBitset> for String {
  fn from(val: &DynamicBitset) -> Self {
    let mut res = String::new();
    val.for_each_bit(|bit| {
      res += if bit != 0 { "1" } else { "0" };
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
      data: vec![0],
      n_bit: 0,
      n_box: 1,
      curr_i_box: 0,
    }
  }
}

#[cfg(test)]
mod test_dynamic_bitset {
  #[test]
  fn basic() {
    use super::DynamicBitset as DBS;

    let mut a = DBS::from("001011");
    assert_eq!(String::from(&a), "001011");

    let b = DBS::from("001011010");
    assert_eq!(String::from(&b), "001011010");

    "010".chars().for_each(|c| a.push(c != '0'));
    assert_eq!(String::from(&a), "001011010");
    assert_eq!(a, b);

    (0..3).for_each(|_| a.pop());
    assert_eq!(String::from(&a), "001011");
    assert_ne!(a, b);
    assert_eq!(a, DBS::from("001011"));
  }
}
