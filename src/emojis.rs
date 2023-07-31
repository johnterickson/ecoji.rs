#[allow(non_snake_case)]
pub struct Version {
    pub VERSION_NUMBER: usize,
    pub PADDING: char,
    pub PADDING_40: char,
    pub PADDING_41: char,
    pub PADDING_42: char,
    pub PADDING_43: char,
    pub EMOJIS: [char; 1024],
    pub EMOJIS_REV: ::phf::Map<char, usize>,
}

include!(concat!(env!("OUT_DIR"), "/emojis.rs"));

impl Version {
    pub fn is_padding(&self, c: char) -> bool {
        [
            self.PADDING,
            self.PADDING_40,
            self.PADDING_41,
            self.PADDING_42,
            self.PADDING_43,
        ]
        .contains(&c)
    }
    pub fn is_valid_alphabet_char(&self, c: char) -> bool {
        self.is_padding(c) || self.EMOJIS_REV.contains_key(&c)
    }
}

#[test]
fn test_mapping() {
    for v in VERSIONS {
        assert_eq!(v.EMOJIS.len(), 1024);
        assert_eq!(v.EMOJIS_REV.len(), 1024);
        for (i, c) in v.EMOJIS.iter().cloned().enumerate() {
            assert_eq!(i, v.EMOJIS_REV[&c]);
        }
    }
}
