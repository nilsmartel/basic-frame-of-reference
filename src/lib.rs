#![feature(int_log)]
use bitvec::vec::BitVec;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub struct FOREncoded(FrameOfReference, BitVec);

impl FOREncoded {
    pub fn encode(values: &[u32]) -> Self {
        let (min, max) = get_size(values);
        let range = max - min;

        // bits needed to encode values 0..range
        let bitlen = range.ilog2() as u8 + 1;
        let offset = min;
        let bitsneeded = values.len() * bitlen as usize;
        let overshoot = (bitsneeded % 8) as u8;

        let fror = FrameOfReference {
            bitlen,
            offset,
            overshoot,
        };

        let mut buf = bitvec::vec::BitVec::with_capacity(bitsneeded);

        for v in values.into_iter().cloned() {
            // adjust to offset
            let v = v - min;

            // now only the first `bitlen` bits are set and needed.

            for i in 0..(bitlen as u32) {
                let bit = v & (1 << i) == 1;
                buf.push(bit);
            }
        }

        FOREncoded(fror, buf)
    }

    pub fn decode(&self) -> Vec<u32> {
        let l = self.len();
        let mut v = Vec::with_capacity(l);

        let offset = self.0.offset;

        let bitlen = self.0.bitlen as usize;
        let mut index = 0;
        for _ in 0..l {
            let bits = self.1.get(index..(index + bitlen)).unwrap();

            let mut value = 0;
            let mut i = 0;
            for b in bits {
                if *b {
                    value |= 1 << i;
                }
                i += 1;
            }

            v.push(value + offset);

            index += bitlen;
        }

        v
    }

    pub fn len(&self) -> usize {
        (self.1.len() - self.0.overshoot as usize) / self.0.bitlen as usize
    }
}

fn get_size(vs: &[u32]) -> (u32, u32) {
    let mut min = u32::MAX;
    let mut max = u32::MIN;

    // This potentially be much faster using simd
    for v in vs {
        min = min.min(*v);
        max = max.max(*v);
    }

    (min, max)
}

struct FrameOfReference {
    offset: u32,
    bitlen: u8,
    // number of overshooting bits
    overshoot: u8,
}
