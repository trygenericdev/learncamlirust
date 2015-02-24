// This is adapted from Camlistore http://camlistore.org/


const WINDOW_SIZE: usize = 64;
const CHAR_OFFSET: usize = 31;

const BLOB_BITS: u32 = 13;
const BLOB_SIZE: u32 = 1 << BLOB_BITS; // 8k

pub struct RollSum {
  s1: u32,
  s2: u32,
  window: [usize; WINDOW_SIZE],
  wofs: usize
}

impl RollSum {

  pub fn new() -> RollSum {
    RollSum{
      s1: WINDOW_SIZE as u32 * CHAR_OFFSET as u32,
      s2: WINDOW_SIZE as u32 * (WINDOW_SIZE as u32 - 1) * CHAR_OFFSET as u32,
      window: [0; WINDOW_SIZE],
      wofs: 0
    }
  }


    fn add(&mut self, drop: usize, add: u8) {
      self.s1 += add as u32 - drop as u32;
      self.s2 += self.s1 - (WINDOW_SIZE as u32) * (drop as u8 + CHAR_OFFSET as u8) as u32;
    }

    pub fn roll(&mut self, ch: u8) {
      let b = self.window[self.wofs];
      self.add(b, ch);
      self.window[self.wofs] = ch as usize;
      self.wofs = (self.wofs + 1) % WINDOW_SIZE;
    }

    // OnSplit returns whether at least 13 consecutive trailing bits of
    // the current checksum are set the same way.
    pub fn on_split(&self) -> bool {
      (self.s2 & (BLOB_SIZE - 1)) == ((!0) & (BLOB_SIZE - 1))
    }

    // OnSplit returns whether at least n consecutive trailing bits
    // of the current checksum are set the same way.
    pub fn on_split_with_bits(&self, n: u32) -> bool {
      let mask = (1u32 << n) - 1;
      self.s2 & mask == (!0u32) & mask
    }

    pub fn bits(&self) -> u32 {
      let mut bits = BLOB_BITS;
      let mut rsum = self.digest();
      rsum = rsum >> BLOB_BITS;
      while (rsum>>1)&1 != 0 {
        rsum = rsum >> 1;
        bits += 1;
      }
      bits
    }

    pub fn digest(&self) -> u32 {
      (self.s1 << 16) | (self.s2 & 0xffff)
    }

}


#[cfg(test)]
mod test {
    extern crate test;
    use self::test::{black_box, Bencher};

    use rollsum::{RollSum, WINDOW_SIZE};

    use std::rand::{thread_rng, Rng};

    #[test]
    fn test_eq() {

      let mut buf = [0u8; 13579];
      thread_rng().fill_bytes(&mut buf);

      let sum = |offset: usize, len: usize| -> u32 {
        let mut rs = RollSum::new();
        for count in offset..len {
          rs.roll(buf[count]);
        }
        return rs.digest();
      };

      let sum1a = sum(0, buf.len());
      let sum1b = sum(1, buf.len());
      let sum2a = sum(buf.len()-WINDOW_SIZE*5/2, buf.len()-WINDOW_SIZE);
      let sum2b = sum(0, buf.len()-WINDOW_SIZE);
      let sum3a = sum(0, WINDOW_SIZE+3);
      let sum3b = sum(3, WINDOW_SIZE+3);

      assert_eq!(sum1a, sum1b);
      assert_eq!(sum2a, sum2b);
      assert_eq!(sum3a, sum3b);
    }

    #[bench]
    fn bench_roll(b: &mut Bencher) {
        let mut buf: Vec<u8> = Vec::with_capacity(5 << 20);
        for i in range(0, 5<<20) {
            buf.push(0u8)
        }
        thread_rng().fill_bytes(buf.as_mut_slice());
        b.iter(|| {
          let mut rs = RollSum::new();
          let mut splits = 0u32;
          let len = buf.len();
          for b in buf.iter() {
            let &bc = b;
            rs.roll(bc);
            if rs.on_split() {
              let bits = rs.bits();
              splits += 1;
            }
          }
          black_box(rs.digest());
        }
      );
    }

}
