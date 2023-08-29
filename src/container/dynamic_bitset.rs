#[allow(dead_code)]
pub struct Info<'a> {
  n_bit: &'a usize,
  n_box: &'a usize,
  curr_i_box: &'a usize,
}

pub struct InfoMut<'a> {
  n_bit: &'a mut usize,
  n_box: &'a mut usize,
  curr_i_box: &'a mut usize,
}

type BoxType = u8;
const BOX_BITS: usize = std::mem::size_of::<BoxType>() * 8;
const USIZE_BYTES: usize = std::mem::size_of::<usize>();

#[derive(Debug, Clone)]
pub struct DynamicBitset {
  data: Vec<BoxType>,
  n_bit: usize,
  n_box: usize,
  curr_i_box: usize,
}

impl std::io::Read for DynamicBitset {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    fn into_le_bytes_arr(num: usize) -> [u8; USIZE_BYTES] {
      let mut res = [0; USIZE_BYTES];
      (0..USIZE_BYTES).for_each(|i| {
        res[i] = (num >> (i * 8)) as u8;
      });
      res
    }
    let Info {
      n_bit,
      n_box,
      curr_i_box,
    } = self.get_info();
    buf[0..USIZE_BYTES].copy_from_slice(&into_le_bytes_arr(*n_bit));
    buf[USIZE_BYTES..USIZE_BYTES * 2].copy_from_slice(&into_le_bytes_arr(*n_box));
    buf[USIZE_BYTES * 2..USIZE_BYTES * 3].copy_from_slice(&into_le_bytes_arr(*curr_i_box));
    buf[USIZE_BYTES * 3..].copy_from_slice(&self.data);
    Ok(buf.len())
  }
}

impl std::io::Write for DynamicBitset {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    fn from_le_bytes_slice(buf: &[u8]) -> usize {
      let mut res = 0;
      (0..buf.len()).for_each(|i| {
        res |= (buf[i] as usize) << (i * 8);
      });
      res
    }
    let InfoMut {
      n_bit,
      n_box,
      curr_i_box,
    } = self.get_info_mut();
    *n_bit = from_le_bytes_slice(&buf[0..USIZE_BYTES]);
    *n_box = from_le_bytes_slice(&buf[USIZE_BYTES..USIZE_BYTES * 2]);
    *curr_i_box = from_le_bytes_slice(&buf[USIZE_BYTES * 2..USIZE_BYTES * 3]);
    self.data = buf[USIZE_BYTES * 3..].to_vec();
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
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
  pub fn sync_with_info(&mut self, info: &InfoMut) {
    self.n_bit = *info.n_bit;
    self.n_box = *info.n_box;
    self.curr_i_box = *info.curr_i_box;
  }

  pub fn get_info_mut(&mut self) -> InfoMut {
    InfoMut {
      n_bit: &mut self.n_bit,
      n_box: &mut self.n_box,
      curr_i_box: &mut self.curr_i_box,
    }
  }

  pub fn get_info(&self) -> Info {
    Info {
      n_bit: &self.n_bit,
      n_box: &self.n_box,
      curr_i_box: &self.curr_i_box,
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
      self.data[self.curr_i_box] |= 1 << (self.n_bit % BOX_BITS);
    } else {
      // `11111111 0 11111111` like
      self.data[self.curr_i_box] &= !(1 << (self.n_bit % BOX_BITS));
    }
    self.n_bit += 1;
  }

  pub fn pop(&mut self) {
    /*
      This function should make sure:
        1. Each unused bit is set to `0`
        2. Each popped bit will be reset to `0`
    */
    self.data[self.curr_i_box] &= !(1 << (self.n_bit % BOX_BITS));
    self.n_bit -= 1;
    self.check_if_sub_box();
  }

  pub fn back(&self) -> bool {
    self.data[self.curr_i_box] & (1 << (self.n_bit % BOX_BITS)) != 0
  }
}

impl DynamicBitset {
  pub fn for_each_bit(&self, mut func: impl FnMut(u8)) {
    // 1. box <- [0..n_box - 1]
    for i in 0..self.n_box - 1 {
      let curr_box = self.data[i];
      for j in 0..BOX_BITS {
        func(curr_box & (1 << j));
      }
    }
    // 2. box <- [n_box - 1]
    let last_box = self.data[self.n_box - 1];
    for j in 0..self.n_bit % BOX_BITS {
      func(last_box & (1 << j));
    }
  }

  pub fn for_each_box(&self, mut func: impl FnMut(u8)) {
    for i in 0..self.n_box {
      func(self.data[i]);
    }
  }
}

impl DynamicBitset {
  fn check_if_add_box(&mut self) {
    let if_add_box = self.n_bit % BOX_BITS == 0 && self.n_bit > 0;
    if !if_add_box {
      return;
    }
    self.data.push(0);
    self.n_box += 1;
    self.curr_i_box += 1;
  }

  fn check_if_sub_box(&mut self) {
    let if_sub_box = self.n_bit % BOX_BITS == 0 && self.n_bit > 1;
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
      data: vec![BoxType::default()],
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
