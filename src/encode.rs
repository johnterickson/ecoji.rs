use crate::emojis::*;
use std::io::{self, Read, Write};
impl Version {
    fn encode_chunk<W: Write + ?Sized>(&self, s: &[u8], out: &mut W) -> io::Result<usize> {
        assert!(!s.is_empty() && s.len() <= 5, "Unexpected slice length");

        let (b0, b1, b2, b3, b4) = (
            s[0] as usize,
            s.get(1).cloned().unwrap_or(0) as usize,
            s.get(2).cloned().unwrap_or(0) as usize,
            s.get(3).cloned().unwrap_or(0) as usize,
            s.get(4).cloned().unwrap_or(0) as usize,
        );

        let mut chars = [
            self.EMOJIS[b0 << 2 | b1 >> 6],
            self.PADDING,
            self.PADDING,
            self.PADDING,
        ];

        match s.len() {
            1 => {}
            2 => chars[1] = self.EMOJIS[(b1 & 0x3f) << 4 | b2 >> 4],
            3 => {
                chars[1] = self.EMOJIS[(b1 & 0x3f) << 4 | b2 >> 4];
                chars[2] = self.EMOJIS[(b2 & 0x0f) << 6 | b3 >> 2];
            }
            4 => {
                chars[1] = self.EMOJIS[(b1 & 0x3f) << 4 | b2 >> 4];
                chars[2] = self.EMOJIS[(b2 & 0x0f) << 6 | b3 >> 2];

                chars[3] = match b3 & 0x03 {
                    0 => self.PADDING_40,
                    1 => self.PADDING_41,
                    2 => self.PADDING_42,
                    3 => self.PADDING_43,
                    _ => unreachable!(),
                }
            }
            5 => {
                chars[1] = self.EMOJIS[(b1 & 0x3f) << 4 | b2 >> 4];
                chars[2] = self.EMOJIS[(b2 & 0x0f) << 6 | b3 >> 2];
                chars[3] = self.EMOJIS[(b3 & 0x03) << 8 | b4];
            }
            _ => unreachable!(),
        }

        let mut buf = [0; 4];
        let mut bytes_written = 0;
        for c in chars.iter() {
            let s = c.encode_utf8(&mut buf).as_bytes();
            out.write_all(s)?;
            bytes_written += s.len();

            if self.VERSION_NUMBER == 2 && self.is_padding(*c) {
                break;
            }
        }

        Ok(bytes_written)
    }

    /// Encodes the entire source into the Ecoji format and writes a UTF-8 representation of
    /// the encoded data to the provided destination.
    ///
    /// If successful, returns the number of bytes which were written to the destination writer.
    ///
    /// Returns an error when either source or destination operation has failed. No guarantees are
    /// made about the state of the destination if an error occurs, so it is possible for the
    /// destination to contain only a part of the encoded data.
    ///
    /// # Examples
    ///
    /// Successful encoding:
    ///
    /// ```
    /// # fn test() -> ::std::io::Result<()> {
    /// let input = "input data";
    ///
    /// let mut output: Vec<u8> = Vec::new();
    /// ecoji::encode(&mut input.as_bytes(), &mut output)?;
    ///
    /// assert_eq!(output, "👶😲🇲👅🍉🔙🌥🌩".as_bytes());
    /// #  Ok(())
    /// # }
    /// # test().unwrap();
    /// ```
    pub fn encode<R: Read + ?Sized, W: Write + ?Sized>(
        &self,
        source: &mut R,
        destination: &mut W,
    ) -> io::Result<usize> {
        let mut buf = [0; 5];
        let mut bytes_written = 0;

        loop {
            let n = read_exact(source, &mut buf)?;

            // EOF
            if n == 0 {
                break;
            }

            bytes_written += self.encode_chunk(&buf[..n], destination)?;
        }

        Ok(bytes_written)
    }

    /// Encodes the entire source into the Ecoji format, storing the result of the encoding to a
    /// new owned string.
    ///
    /// Returns a string with the encoded data if successful.
    ///
    /// Failure conditions are exactly the same as those of the [`encode`](fn.encode.html) function;
    /// because the encoding output is always a valid sequence of emoji code points, it is guaranteed
    /// to be representable as a valid UTF-8 sequence.
    ///
    /// # Examples
    ///
    /// Successful encoding:
    ///
    /// ```
    /// # fn test() -> ::std::io::Result<()> {
    /// let input = "input data";
    /// let output: String = ecoji::encode_to_string(&mut input.as_bytes())?;
    ///
    /// assert_eq!(output, "👶😲🇲👅🍉🔙🌥🌩");
    /// #  Ok(())
    /// # }
    /// # test().unwrap();
    /// ```
    pub fn encode_to_string<R: Read + ?Sized>(&self, source: &mut R) -> io::Result<String> {
        let mut output = Vec::new();
        self.encode(source, &mut output)?;
        // encoded output is guaranteed to be valid UTF-8
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }
}

fn read_exact<R: Read + ?Sized>(source: &mut R, mut buf: &mut [u8]) -> io::Result<usize> {
    let mut bytes_read = 0;
    while !buf.is_empty() {
        match source.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
                bytes_read += n;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(bytes_read)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(v: &Version, input: &[u8], output: &[u8]) {
        let encoded = v.encode_to_string(&mut input.clone()).unwrap();
        dbg!(output.len());
        dbg!(std::str::from_utf8(output).unwrap());
        dbg!(encoded.as_bytes().len());
        dbg!(&encoded);
        assert_eq!(output, encoded.as_bytes());
    }

    fn check_chars(v: &Version, input: &[u8], output: &[char]) {
        let buf = v.encode_to_string(&mut input.clone()).unwrap();
        let chars: Vec<_> = buf.chars().collect();
        let mut output: Vec<_> = output.iter().cloned().collect();
        while v.VERSION_NUMBER > 1
            && output.get(output.len() - 2..output.len()) == Some(&[v.PADDING, v.PADDING])
        {
            output.pop();
        }
        assert_eq!(output, chars.as_slice());
    }

    fn check_all(input: &[u8], output: &[&[u8]]) {
        for (i, v) in VERSIONS.iter().enumerate() {
            dbg!(v.VERSION_NUMBER);
            check(v, input, &output[i]);
        }
    }

    #[test]
    fn test_random() {
        check_all(b"abc", &["👖📸🎈☕".as_bytes(), "👖📸🎈☕".as_bytes()]);
    }

    #[test]
    fn test_one_byte() {
        for v in VERSIONS {
            check_chars(
                v,
                b"k",
                &[
                    v.EMOJIS[('k' as usize) << 2],
                    v.PADDING,
                    v.PADDING,
                    v.PADDING,
                ],
            );
        }
    }

    #[test]
    fn test_two_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[0, 1],
                &[v.EMOJIS[0], v.EMOJIS[16], v.PADDING, v.PADDING],
            );
        }
    }

    #[test]
    fn test_three_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[0, 1, 2],
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING],
            );
        }
    }

    #[test]
    fn test_four_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[0, 1, 2, 0],
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_40],
            );
            check_chars(
                v,
                &[0, 1, 2, 1],
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_41],
            );
            check_chars(
                v,
                &[0, 1, 2, 2],
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_42],
            );
            check_chars(
                v,
                &[0, 1, 2, 3],
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_43],
            );
        }
    }

    #[test]
    fn test_five_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[0xAB, 0xCD, 0xEF, 0x01, 0x23],
                &[v.EMOJIS[687], v.EMOJIS[222], v.EMOJIS[960], v.EMOJIS[291]],
            );
        }
    }
}
