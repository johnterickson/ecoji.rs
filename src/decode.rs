use std::io::{self, Read, Write};

use crate::chars::{Chars, CharsError};
use crate::emojis::*;

impl Version {
    /// Decodes the entire source from the Ecoji format (assumed to be UTF-8-encoded) and writes the
    /// result of the decoding to the provided destination.
    ///
    /// If successful, returns the number of bytes which were written to the destination writer.
    ///
    /// Returns an error when either source or destination operation has failed, if the number of
    /// code points in the input is wrong (it must be a multiple of 4), if the source is not
    /// a valid UTF-8 stream or if one of the code points in the source is not a valid character
    /// of the Ecoji alphabet. No guarantees are made about the state of the destination if an error
    /// occurs, so it is possible for the destination to contain only a part of the decoded data.
    ///
    /// # Examples
    ///
    /// Successful decoding:
    ///
    /// ```
    /// # fn test() -> ::std::io::Result<()> {
    /// let input = "👶😲🇲👅🍉🔙🌥🌩";
    ///
    /// let mut output: Vec<u8> = Vec::new();
    /// ecoji::decode(&mut input.as_bytes(), &mut output)?;
    ///
    /// assert_eq!(output, b"input data");
    /// #  Ok(())
    /// # }
    /// # test().unwrap();
    /// ```
    ///
    /// Invalid input data, not enough code points:
    ///
    /// ```
    /// use std::io;
    ///
    /// let input = "👶😲🇲👅🍉🔙🌥";  // one less than needed
    ///
    /// let mut output: Vec<u8> = Vec::new();
    /// match ecoji::decode(&mut input.as_bytes(), &mut output) {
    ///     Ok(_) => panic!("Unexpected success"),
    ///     Err(e) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
    /// }
    /// ```
    ///
    /// Invalid input data, not a correct UTF-8 stream:
    ///
    /// ```
    /// use std::io;
    ///
    /// let input: &[u8] = &[0xfe, 0xfe, 0xff, 0xff];
    ///
    /// let mut output: Vec<u8> = Vec::new();
    /// match ecoji::decode(&mut input.clone(), &mut output) {
    ///     Ok(_) => panic!("Unexpected success"),
    ///     Err(e) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
    /// }
    /// ```
    ///
    /// Invalid input data, input code point is not a part of the Ecoji alphabet:
    ///
    /// ```
    /// use std::io;
    ///
    /// // Padded with spaces for the length to be a multiple of 4
    /// let input = "Not emoji data  ";
    ///
    /// let mut output: Vec<u8> = Vec::new();
    /// match ecoji::decode(&mut input.as_bytes(), &mut output) {
    ///     Ok(_) => panic!("Unexpected success"),
    ///     Err(e) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
    /// }
    /// ```
    pub fn decode<R: Read + ?Sized, W: Write + ?Sized>(
        &self,
        source: &mut R,
        destination: &mut W,
    ) -> io::Result<usize> {
        let mut input = Chars::new(source);

        let mut bytes_written = 0;
        let mut decoder = self;
        loop {
            let mut chars = ['\0'; 4];

            match input.next() {
                Some(c) => chars[0] = self.check_char(&mut decoder, c)?,
                None => break,
            };

            let mut last_was_padding = false;
            for chars in chars.iter_mut().skip(1) {
                match input.next() {
                    Some(c) => {
                        let c = self.check_char(&mut decoder, c)?;
                        last_was_padding = decoder.is_padding(c);
                        *chars = c;
                    }
                    None => {
                        if !last_was_padding {
                            return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "Unexpected end of data, input code points count is not a multiple of 4"));
                        }
                    }
                }
            }

            let (bits1, bits2, bits3) = (
                decoder.EMOJIS_REV.get(&chars[0]).cloned().unwrap_or(0),
                decoder.EMOJIS_REV.get(&chars[1]).cloned().unwrap_or(0),
                decoder.EMOJIS_REV.get(&chars[2]).cloned().unwrap_or(0),
            );
            let bits4 = if chars[3] == decoder.PADDING_40 {
                0
            } else if chars[3] == decoder.PADDING_41 {
                1 << 8
            } else if chars[3] == decoder.PADDING_42 {
                2 << 8
            } else if chars[3] == decoder.PADDING_43 {
                3 << 8
            } else {
                decoder.EMOJIS_REV.get(&chars[3]).cloned().unwrap_or(0)
            };

            let out = [
                (bits1 >> 2) as u8,
                (((bits1 & 0x3) << 6) | (bits2 >> 4)) as u8,
                (((bits2 & 0xf) << 4) | (bits3 >> 6)) as u8,
                (((bits3 & 0x3f) << 2) | (bits4 >> 8)) as u8,
                (bits4 & 0xff) as u8,
            ];

            let out = if chars[1] == decoder.PADDING {
                &out[..1]
            } else if chars[2] == decoder.PADDING {
                &out[..2]
            } else if chars[3] == decoder.PADDING {
                &out[..3]
            } else if chars[3] == decoder.PADDING_40
                || chars[3] == decoder.PADDING_41
                || chars[3] == decoder.PADDING_42
                || chars[3] == decoder.PADDING_43
            {
                &out[..4]
            } else {
                &out[..]
            };

            destination.write_all(out)?;
            bytes_written += out.len();
        }

        Ok(bytes_written)
    }

    /// Decodes the entire source from the Ecoji format (assumed to be UTF-8-encoded), storing the
    /// result of the decoding to a new byte vector.
    ///
    /// Returns a byte vector with the decoded data if successful.
    ///
    /// Failure conditions are exactly the same as those of the [`decode`](fn.decode.html) function.
    ///
    /// # Examples
    ///
    /// Successful decoding:
    ///
    /// ```
    /// # fn test() -> ::std::io::Result<()> {
    /// let input = "👶😲🇲👅🍉🔙🌥🌩";
    /// let output: Vec<u8> = ecoji::decode_to_vec(&mut input.as_bytes())?;
    ///
    /// assert_eq!(output, b"input data");
    /// #  Ok(())
    /// # }
    /// # test().unwrap();
    /// ```
    ///
    /// See [`decode`](fn.decode.html) docs for error examples.
    pub fn decode_to_vec<R: Read + ?Sized>(&self, source: &mut R) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        self.decode(source, &mut output)?;
        Ok(output)
    }

    /// Decodes the entire source from the Ecoji format (assumed to be UTF-8-encoded), storing the
    /// result of the decoding to a new owned string.
    ///
    /// Returns a string with the decoded data if successful.
    ///
    /// In addition to the [`decode`](fn.decode.html) failure conditions, this function also returns
    /// an error if the decoded data is not a valid UTF-8 string.
    ///
    /// # Examples
    ///
    /// Successful decoding:
    ///
    /// ```
    /// # fn test() -> ::std::io::Result<()> {
    /// let input = "👶😲🇲👅🍉🔙🌥🌩";
    /// let output: String = ecoji::decode_to_string(&mut input.as_bytes())?;
    ///
    /// assert_eq!(output, "input data");
    /// #  Ok(())
    /// # }
    /// # test().unwrap();
    /// ```
    ///
    /// Invalid input data, decoded string is not a valid UTF-8 string:
    ///
    /// ```
    /// use std::io;
    ///
    /// let input = "🧑🦲🧕🙋";  // Encoded data: [0xfe, 0xfe, 0xff, 0xff]
    ///
    /// match ecoji::decode_to_string(&mut input.as_bytes()) {
    ///     Ok(_) => panic!("Unexpected success"),
    ///     Err(e) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
    /// }
    /// ```
    pub fn decode_to_string<R: Read + ?Sized>(&self, source: &mut R) -> io::Result<String> {
        let output = self.decode_to_vec(source)?;
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn check_char(&self, decoder: &mut &Version, c: Result<char, CharsError>) -> io::Result<char> {
        c.map_err(CharsError::into_io).and_then(|c| {
            if decoder.is_valid_alphabet_char(c) {
                return Ok(c);
            } else {
                // switch to the other decoder if we've not already
                if std::ptr::eq(self, *decoder) {
                    *decoder = self.other_version();
                    if decoder.is_valid_alphabet_char(c) {
                        return Ok(c);
                    }
                }
            }

            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Input character '{}' is not a part of the Ecoji alphabet",
                    c
                ),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(v: &Version, input: &[u8], output: &[u8]) {
        let buf = v.decode_to_vec(&mut input.clone()).unwrap();
        assert_eq!(output, buf.as_slice());
    }

    fn check_all(input: &[&[u8]], output: &[u8]) {
        for v in VERSIONS {
            for input in input {
                check(v, input, output);
            }
        }
    }

    fn check_chars(v: &Version, input: &[char], output: &[u8]) {
        let input: String = input.iter().cloned().collect();
        let buf = v.decode_to_vec(&mut input.as_bytes()).unwrap();
        assert_eq!(output, buf.as_slice());
    }

    #[test]
    fn test_random() {
        check_all(&["👖📸🎈☕".as_bytes(), "👖📸🎈☕".as_bytes()], b"abc");
    }

    #[test]
    fn test_one_byte() {
        for v in VERSIONS {
            check_chars(
                v,
                &[
                    v.EMOJIS[('k' as usize) << 2],
                    v.PADDING,
                    v.PADDING,
                    v.PADDING,
                ],
                b"k",
            );
        }
    }

    #[test]
    fn test_two_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.PADDING, v.PADDING],
                &[0, 1],
            );
        }
    }

    #[test]
    fn test_three_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING],
                &[0, 1, 2],
            );
        }
    }

    #[test]
    fn test_four_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_40],
                &[0, 1, 2, 0],
            );
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_41],
                &[0, 1, 2, 1],
            );
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_42],
                &[0, 1, 2, 2],
            );
            check_chars(
                v,
                &[v.EMOJIS[0], v.EMOJIS[16], v.EMOJIS[128], v.PADDING_43],
                &[0, 1, 2, 3],
            );
        }
    }

    #[test]
    fn test_five_bytes() {
        for v in VERSIONS {
            check_chars(
                v,
                &[v.EMOJIS[687], v.EMOJIS[222], v.EMOJIS[960], v.EMOJIS[291]],
                &[0xAB, 0xCD, 0xEF, 0x01, 0x23],
            );
        }
    }
}
