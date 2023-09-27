use anchor_lang::prelude::*;

// Storage space must be known in advance, as such all strings are limited to 64 chars
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub struct LimitedString {
    pub value: [u8; 64], // Self::MaxLenght - anchor bug, cannot use constants here
    pub length: u8,
}

impl PartialEq for LimitedString {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length
            && self.value[..self.length as usize] == other.value[..other.length as usize]
    }
}

impl From<LimitedString> for String {
    fn from(limited_string: LimitedString) -> Self {
        let mut string = String::new();
        for byte in limited_string.value.iter() {
            if *byte == 0 {
                break;
            }
            string.push(*byte as char);
        }
        string
    }
}

impl LimitedString {
    pub fn to_bytes(&self) -> &[u8] {
        &self.value[..self.length as usize]
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.value[..self.length as usize]).into_owned()
    }
}

impl Default for LimitedString {
    fn default() -> Self {
        Self {
            value: [0; Self::MAX_LENGTH],
            length: 0,
        }
    }
}

impl LimitedString {
    pub const MAX_LENGTH: usize = 64;

    pub fn new<S: AsRef<str>>(input: S) -> Self {
        let input_str = input.as_ref();
        let length = input_str.len() as u8;
        let mut array = [0; Self::MAX_LENGTH];
        let bytes = input_str.as_bytes();
        for (index, byte) in bytes.iter().enumerate() {
            if index >= Self::MAX_LENGTH {
                break;
            }
            array[index] = *byte;
        }
        LimitedString {
            value: array,
            length,
        }
    }
}
