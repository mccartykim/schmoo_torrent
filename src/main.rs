// assume only valid bencoding, only one top level object
mod bencode {
    use std::collections::HashMap;
    use std::fmt;
    use crate::bencode::Bencode::*;
    pub enum Bencode {
        BList(Vec<Bencode>),
        BString(String),
        BDict(HashMap<String, Bencode>),
        BInt(i64),
    }

    impl Bencode {
        fn encode(&self) -> String {
            match self {
                BString(string) => format!("{}:{}", string.len(), string),
                BInt(int) => format!("i{}e", int),
                BList(list) => _encode_list(list),
                BDict(dict) => _encode_dict(dict),
            }
        }

        fn decode(string: &str) -> (Bencode, &str) {
            let mut chars = string.chars();

            let starting_char = chars.next();
            println!(
                "starting char: {}, full string: {}",
                starting_char.unwrap().to_string(),
                chars.as_str()
            );

            let result = match starting_char {
                Some('l') => _decode_list(chars.as_str()),
                Some('d') => _decode_dict(chars.as_str()),
                Some('i') => _decode_int(chars.as_str()),
                None => panic!("unexpected string end"),
                Some(_) => _decode_b_string(string),
            };

            result
        }
    }

    impl fmt::Debug for Bencode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                BList(list) => write!(f, "{:?}", list),
                BDict(dict) => write!(f, "{:?}", dict),
                BInt(val) => write!(f, "{:?}", val),
                BString(val) => write!(f, "{:?}", val),
            }
        }
    }

    impl PartialEq for Bencode {
        fn eq(&self, other: &Bencode) -> bool {
            match self {
                Bencode::BString(value) => {
                    if let Bencode::BString(other_val) = other {
                        value == other_val
                    } else {
                        false
                    }
                },
                Bencode::BList(value) => {
                    if let Bencode::BList(other_val) = other {
                        value == other_val
                    } else {
                        false
                    }
                },
                Bencode::BDict(value) => {
                    if let Bencode::BDict(other_val) = other {
                        value == other_val
                    } else {
                        false
                    }
                },
                Bencode::BInt(value) => {
                    if let Bencode::BInt(other_val) = other {
                        value == other_val
                    } else {
                        false
                    }
                }
            }
        }
    }

    fn _decode_list(string: &str) -> (Bencode, &str) {
        let mut results: Vec<Bencode> = Vec::new();
        let mut substr = string;
        let mut cursor = string.chars().next();
        return loop {
            match cursor {
                Some('e') => break (BList(results), substr),
                None => panic!("String terminated unexpectedly"),
                _ => {
                    let result = Bencode::decode(substr);
                    results.push(result.0);
                    substr = result.1;
                    cursor = substr.chars().next()
                }
            }
        };
    }

    fn _decode_dict(string: &str) -> (Bencode, &str) {
        // ugh i need to think hard in recursion don't I?
        // decode a string and start slice after string
        // send slice to main decode method, and then take the end cursor
        // if end cursor is 'e', done
        let mut results: HashMap<String, Bencode> = HashMap::new();
        let mut chars = string.chars();
        let mut cursor = Some('0'); // placeholder, so we don't advance the real cursor
        return loop {
            match cursor {
                Some('e') => break (BDict(results), chars.as_str()),
                None => panic!("String terminated unexpectedly"),
                _ => {
                    let (key, remainder) = _decode_string(chars.as_str());
                    let (value, remainder) = Bencode::decode(remainder);
                    results.insert(String::from(key), value);
                    chars = remainder.chars();
                    cursor = chars.next();
                }
            }
        };
    }

    fn _decode_int(string: &str) -> (Bencode, &str) {
        let end = string.find('e').unwrap();
        let int = string[..end].parse::<i64>().unwrap();
        return (BInt(int), &string[end + 1..]);
    }

    fn _decode_string(string: &str) -> (&str, &str) {
        let delimiter = string.find(':').unwrap();
        let length = string[..delimiter].parse::<usize>().unwrap();
        let word_start = delimiter + 1;
        let word_end = word_start + length;

        return (&string[word_start..word_end], &string[word_end..]);
    }

    fn _decode_b_string(string: &str) -> (Bencode, &str) {
        let delimiter = string.find(':').unwrap();
        let length = string[..delimiter].parse::<usize>().unwrap();
        let word_start = delimiter + 1;
        let word_end = word_start + length;

        return (
            BString(string[word_start..word_end].to_string()),
            &string[word_end..],
        );
    }

    fn _encode_list(list: &Vec<Bencode>) -> String {
        let mut result = String::from("l");
        for obj in list.iter() {
            result.push_str(&obj.encode())
        }
        result.push_str("e");
        result
    }

    fn _encode_dict(dict: &HashMap<String, Bencode>) -> String {
        let mut result = String::from("d");
        for (key, value) in dict {
            result.push_str(&format!("{}:{}", key.len(), key));
            result.push_str(&value.encode());
        }
        result.push_str("e");
        result
    }

    mod tests {
        use crate::bencode::Bencode;
        use crate::bencode::Bencode::*;
        use std::collections::HashMap;

        #[test]
        fn encodes_string() {
            assert_eq!(
                BString("hamburger".to_string()).encode(),
                "9:hamburger"
            );
        }

        #[test]
        fn decodes_string() {
            assert_eq!(
                Bencode::decode("9:hamburger").0,
                BString("hamburger".to_string())
            );
        }

        #[test]
        fn encodes_int() {
            assert_eq!(BInt(10).encode(), "i10e");
        }

        #[test]
        fn encodes_empty_list() {
            assert_eq!(BList(Vec::new()).encode(), "le");
        }

        #[test]
        fn encodes_list_of_one() {
            assert_eq!(BList(vec![BInt(1)]).encode(), "li1ee");
        }

        #[test]
        fn encodes_list_of_int_and_string() {
            assert_eq!(
                BList(vec![BInt(1), BString(String::from("ace"))]).encode(),
                "li1e3:acee"
            );
        }

        #[test]
        fn encodes_list_of_string_and_int() {
            assert_eq!(
                BList(vec![BString(String::from("ace")), BInt(1)]).encode(),
                "l3:acei1ee"
            );
        }

        #[test]
        fn encodes_empty_dict() {
            assert_eq!(BDict(HashMap::new()).encode(), "de");
        }

        #[test]
        fn encodes_dict() {
            let mut dict = HashMap::new();
            dict.insert("test".to_string(), BInt(1));
            assert_eq!(BDict(dict).encode(), "d4:testi1ee");
        }

        #[test]
        fn encodes_dict_with_empty_list() {
            let mut dict = HashMap::new();
            dict.insert("test".to_string(), BList(vec![]));
            assert_eq!(BDict(dict).encode(), "d4:testlee");
        }
    }
}

mod decode_metainfo {
    use maplit::hashmap;
    use crate::bencode::Bencode;

    fn decode_metainfo() -> Bencode {
        // TODO actual implementation
        return Bencode::BDict(hashmap!{});
    }
}

mod tracker_communication {
    struct Request {
        info_hash: String, // TODO is there a better SHA 1 byte string representation?
        peer_id: String,
        port: usize,
        uploaded: String,
        downloaded: String,
        left: String,
        compact: bool, // true should become 1, false should become 0
        no_peer_id: Option<bool>,
        event: Option<TorrentEvent>,
        ip: Option<String>,
        numwant: Option<usize>,
        key: Option<String>,
        trackerid: Option<String>
    }

    enum TorrentEvent {
        Started,
        Stopped,
        Completed
    }

    // TODO response

    // TODO scrape
}

mod peer_protocol {

    struct PeerState {
        am_choking: bool,
        choking_me: bool,
        am_interested: bool,
        interested_in_me: bool
    }

    static INITIAL_STATE: PeerState = PeerState {
        am_choking: true,
        am_interested: false,
        choking_me: true,
        interested_in_me: false
    };

    // nb: integers are 4 byte big endian values

    struct Handshake {
        pstrlen: u8,
        pstr: String,
        reserved: u64,
        info_hash: String, // TODO is this the correct hash encoding?
        peer_id: String
    }

    enum Messages {
        KeepAlive,
        Choke,
        Unchoke,
        Interested,
        NotInterested,
        Have(Have),
        Bitfield(Raw_Bitfield),
        Request(Request),
        Piece(Piece),
        Cancel(Cancel),
        Port(String)
    }

    struct Have { piece_index: u32 }
    // TODO should this carry the bitfield object we made maybe?
    struct Raw_Bitfield { bytes: Vec<u8> }
    struct Request { index: u32, begin: u32, length: u32 }
    struct Piece { index: u32, begin: u32, block: Vec<u8> }
    struct Cancel { index: u32, begin: u32, length: u32 }
}

mod pieces {
    use crate::bitfield::Bitfield;
    use sha1::{Sha1, Digest};

    struct Pieces {
        bitfield: Bitfield,
        pieces: Vec<Piece>
    }

    impl Pieces {

    }

    struct Piece {
        sha: [u8; 20],
        index: u32,
        bytes: Vec<u8>,
        _done: bool
    }

    impl Piece {
        fn is_done(&mut self) -> bool {
            if !self._done {
                let mut hasher = Sha1::new();
                hasher.input(&self.bytes);
                let result = hasher.result();
                self._done = result[..] == self.sha;
            }
            self._done
        }
    }
}

mod bitfield {
    pub struct Bitfield {
        len: usize,
        bytes: Vec<u8>
    }

    impl Bitfield {

        fn create(bit_size: usize) -> Bitfield {
            let byte_size = (bit_size/8) + (if bit_size % 8 != 0 {1} else {0});
            return Bitfield { len: bit_size, bytes: vec![0; byte_size] };
        }

        fn set_value(&mut self, index: usize) {
            let byte_index = index / 8;
            let bit_index = index % 8;

            let bit_twiddle = 0 | (1 << bit_index);

            self.bytes[byte_index] = self.bytes[byte_index] | bit_twiddle;
        }

        fn unset_value(&mut self, index: usize) {
            let byte_index = index / 8;
            let bit_index = index % 8;

            let bit_twiddle = 0b11111111 ^ (1 << bit_index);

            self.bytes[byte_index] = self.bytes[byte_index] & bit_twiddle;
        }

        fn get_value(&self, index: usize) -> bool {
            let byte_index = index / 8;
            let bit_index = index % 8;

            let mask = 1 << bit_index;

            return (self.bytes[byte_index] & mask) != 0;
        }
    }

    mod tests {
        use crate::bitfield::Bitfield;
        #[test]
        fn one_byte_is_all_false() {
            let vals = Bitfield::create(8);
            assert_eq!(vals.get_value(0), false);
            assert_eq!(vals.get_value(7), false);
        }

        #[test]
        fn one_byte_can_set_values() {
            let mut vals = Bitfield::create(8);
            vals.set_value(2);
            assert_eq!(vals.get_value(2), true);
            vals.unset_value(2);
            assert_eq!(vals.get_value(2), false);
        }

        #[test]
        fn less_than_one_byte_can_set_values() {
            let mut vals = Bitfield::create(3);
            vals.set_value(2);
            assert_eq!(vals.get_value(2), true);
            vals.unset_value(2);
            assert_eq!(vals.get_value(2), false);
        }

        #[test]
        fn multiple_byte_can_set_values() {
            let mut vals = Bitfield::create(64);
            vals.set_value(19);
            assert_eq!(vals.get_value(19), true);
            vals.unset_value(19);
            assert_eq!(vals.get_value(19), false);
            vals.set_value(63);
            assert_eq!(vals.get_value(63), true);
            vals.unset_value(63);
            assert_eq!(vals.get_value(63), false);
        }
    }
}
