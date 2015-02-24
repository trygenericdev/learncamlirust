
extern crate camli;

use std::old_io::{BufferedReader, File};

use camli::rollsum::RollSum;

fn main() {
  let path = Path::new("testfile");
  let mut file = BufferedReader::new(File::open(&path));
  let mut rs = RollSum::new();

  let mut splits = 0u32;
  loop {
      let b = file.read_byte();
      match b {
        Ok(cb) => {
            rs.roll(cb);
            if rs.on_split() {
              //println!("{} - {}", rs.bits(), rs.digest());
              splits += 1;
            }
          }
        Err(e) => {
          break;
          }
      }
  }
  println!("Splits: {}", splits)
}
