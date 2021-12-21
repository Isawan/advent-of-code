use byteorder::{BigEndian, ByteOrder};
use hex::FromHex;
use std::fs;
use structopt::StructOpt;
use std::fmt::{Formatter, Error, Display};

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Payload {
    Literal(u64),
    Operator(Vec<Box<Packet>>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct Packet {
    version: u8,
    type_id: u8,
    payload: Payload,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct BitSlice<'a> {
    bytes: &'a [u8],
    /// bit offset into bytes from the start
    offset: u8,
}

impl<'a> BitSlice<'a> {
    fn new(bytes: &'a [u8], offset: u8) -> Self {
        BitSlice {
            bytes: &bytes[(offset / 8) as usize..],
            offset: offset % 8,
        }
    }
    fn advance(&self, bits: usize) -> Self {
        let offset = (self.offset as usize) + bits;
        let bytes = self.bytes;
        BitSlice {
            bytes: &bytes[(offset / 8)..],
            offset: (offset % 8) as u8,
        }
    }
    fn len(&self) -> usize {
        self.bytes.len() * 8 - (self.offset as usize)
    }
}

impl Display for BitSlice<'_>{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{{ offset: {}, ", self.offset);
        for b in self.bytes {
            write!(f, "{:08b}", b);
        }
        write!(f, " }}")
    }

}

fn parse_version(stream: BitSlice) -> (u8, BitSlice) {
    let mut buffer = [0, 0];
    buffer[0] = stream.bytes[0];
    buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
    let extract = BigEndian::read_u16(&buffer);
    let masked = (0b1110_0000_0000_0000 >> stream.offset) & extract;
    let number = masked >> ((16 - 3) - stream.offset);
    (number as u8, BitSlice::new(stream.bytes, stream.offset + 3))
}

fn parse_type_id(stream: BitSlice) -> (u8, BitSlice) {

    let mut buffer = [0, 0];
    buffer[0] = stream.bytes[0];
    buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
    let extract = BigEndian::read_u16(&buffer);
    let masked = (0b1110_0000_0000_0000 >> stream.offset) & extract;
    let number = masked >> ((16 - 3) - stream.offset);
    (number as u8, BitSlice::new(stream.bytes, stream.offset + 3))
}

fn parse_literal(mut stream: BitSlice) -> (u64, BitSlice) {
    let mut total: u64 = 0;
    let mut buffer = [0, 0];

    // get number of loops
    let mut lead_mask: u16 = 0b1000_0000_0000_0000 >> stream.offset;
    let mut digit_mask: u16 = 0b0111_1000_0000_0000 >> stream.offset;
    loop {
        buffer[0] = stream.bytes[0];
        buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
        let extract = BigEndian::read_u16(&buffer);
        let number = (extract & digit_mask) >> ((16 - 5) - stream.offset);
        total = total * 16;
        total = total | number as u64;
        if extract & lead_mask == 0 {
            stream = BitSlice::new(stream.bytes, stream.offset + 5);
            break;
        }

        digit_mask = digit_mask.rotate_right(5);
        lead_mask = lead_mask.rotate_right(5);

        if (stream.offset + 5) / 8 != 0 {
            digit_mask = digit_mask.rotate_right(8);
            lead_mask = lead_mask.rotate_right(8);
        }
        stream = BitSlice::new(stream.bytes, stream.offset + 5);
    }
    (total, stream)
}

fn parse_length_type_0_packets(mut stream: BitSlice) -> (Vec<Box<Packet>>, BitSlice) {
    let mut buffer = [0, 0, 0 ,0];
    buffer[0] = stream.bytes[0];
    buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
    buffer[2] = *stream.bytes.get(2).unwrap_or(&0);
    buffer[3] = *stream.bytes.get(3).unwrap_or(&0);

    let extract = BigEndian::read_u32(&buffer);
    let masked = (0b1111_1111_1111_1110_0000_0000_0000_0000 >> stream.offset) & extract;
    let length = (masked >> ((32 - 15) - stream.offset)) as usize;
    stream = stream.advance(15);
    let mut packets = Vec::new();
    assert!(stream.len() >= length);
    let old_stream_len = stream.len();
    while old_stream_len - stream.len() != length {
        let result = parse_packet(stream);
        packets.push(Box::new(result.0));
        stream = result.1;
    }
    (packets, stream)
}

fn parse_length_type_1_packets(mut stream: BitSlice) -> (Vec<Box<Packet>>, BitSlice) {
    let mut buffer = [0, 0, 0, 0];
    buffer[0] = stream.bytes[0];
    buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
    buffer[2] = *stream.bytes.get(2).unwrap_or(&0);
    buffer[3] = *stream.bytes.get(3).unwrap_or(&0);

    let extract = BigEndian::read_u32(&buffer);
    let masked = (0b1111_1111_1110_0000_0000_0000_0000_0000 >> stream.offset) & extract;
    let count = (masked >> ((32 - 11) - stream.offset)) as usize;
    let mut packets = Vec::new();
    stream = stream.advance(11);
    for _ in 0..count {
        let result = parse_packet(stream);
        packets.push(Box::new(result.0));
        stream = result.1;
    }
    (packets, stream)
}

fn parse_length_type(mut stream: BitSlice) -> (usize, BitSlice) {
    let mut buffer = [0, 0, 0, 0];
    buffer[0] = stream.bytes[0];
    buffer[1] = *stream.bytes.get(1).unwrap_or(&0);
    buffer[2] = *stream.bytes.get(2).unwrap_or(&0);
    buffer[3] = *stream.bytes.get(3).unwrap_or(&0);

    let extract = BigEndian::read_u32(&buffer);
    let masked = (0b1000_0000_0000_0000_0000_0000_0000_0000 >> stream.offset) & extract;
    let length_type = (masked >> ((32 - 1) - stream.offset)) as usize;
    stream = stream.advance(1);
    (length_type, stream)
}

fn parse_packet(mut stream: BitSlice) -> (Packet, BitSlice) {
    let (version, stream) = parse_version(stream);
    let (type_id, mut stream) = parse_type_id(stream);
    let payload = match type_id {
        4 => {
            let result = parse_literal(stream);
            let literal = result.0;
            stream = result.1;
            Payload::Literal(literal)
        }
        _ => {
            let result = parse_length_type(stream);
            let length_type = result.0;
            stream = result.1;
            Payload::Operator(match length_type {
                0 => {
                    let result = parse_length_type_0_packets(stream);
                    stream = result.1;
                    result.0
                }
                1 => {
                    let result = parse_length_type_1_packets(stream);
                    stream = result.1;
                    result.0
                }
                _ => {
                    panic!("unexpected length type");
                }
            })
        }
    };
    (
        Packet {
            version,
            type_id,
            payload,
        },
        stream,
    )
}

fn sum_version_numbers(packet: &Packet) -> u64 {
    (packet.version as u64)
        + match &packet.payload {
            Payload::Literal(_) => 0,
            Payload::Operator(v) => v.iter().fold(0, |a, x| a + sum_version_numbers(&x)),
        }
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let stream: Vec<u8> = FromHex::from_hex(source.trim()).unwrap();
    let (packet,_) = parse_packet(BitSlice::new(&stream, 0));
    let sum = sum_version_numbers(&packet);
    println!("sum of version numbers: {}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_slice() {
        let slice = Vec::from([0, 1, 2, 3, 4, 5, 6, 7]);
        let offset = 0;
        let bitslice = BitSlice::new(&slice, offset);
        assert_eq!(bitslice.bytes, slice);
        assert_eq!(bitslice.offset, 0);

        let slice = Vec::from([0, 1, 2, 3, 4, 5, 6, 7]);
        let offset = 6;
        let bitslice = BitSlice::new(&slice, offset);
        assert_eq!(bitslice.bytes, slice);
        assert_eq!(bitslice.offset, 6);

        let slice = Vec::from([0, 1, 2, 3, 4, 5, 6, 7]);
        let offset = 8;
        let bitslice = BitSlice::new(&slice, offset);
        assert_eq!(bitslice.bytes, &slice[1..]);
        assert_eq!(bitslice.offset, 0);

        let slice = Vec::from([0, 1, 2, 3, 4, 5, 6, 7]);
        let offset = 16;
        let bitslice = BitSlice::new(&slice, offset);
        assert_eq!(bitslice.bytes, &slice[2..]);
        assert_eq!(bitslice.offset, 0);
    }

    #[test]
    fn test_version_number() {
        let slice = Vec::from([0b1101_0010, 0b1111_1110]);
        let offset = 0;
        let (version, bits) = parse_version(BitSlice::new(&slice, offset));
        assert_eq!(version, 6);
        assert_eq!(bits.bytes, slice);
        assert_eq!(bits.offset, 3);

        let slice = Vec::from([0b1101_0010, 0b1111_1110]);
        let offset = 6;
        let (version, bits) = parse_version(BitSlice::new(&slice, offset));
        assert_eq!(version, 5);
        assert_eq!(bits.bytes, &slice[1..]);
        assert_eq!(bits.offset, 1);
    }

    #[test]
    fn test_type_id() {
        let slice = Vec::from([0b1101_0010, 0b1111_1110]);
        let offset = 3;
        let (type_id, bits) = parse_type_id(BitSlice::new(&slice, offset));
        assert_eq!(type_id, 4);
        assert_eq!(bits.bytes, slice);
        assert_eq!(bits.offset, 6);
    }

    #[test]
    fn test_parse_literal() {
        let slice = Vec::from([0b1101_0010, 0b1111_1110, 0b0010_1000]);
        let offset = 6;
        let (literal, bits) = parse_literal(BitSlice::new(&slice, offset));
        assert_eq!(literal, 2021);
        assert_eq!(bits.bytes, &slice[2..]);
        assert_eq!(bits.offset, 5);
    }

    #[test]
    fn test_parse_literal_packet() {
        let slice = Vec::from([0b1101_0010, 0b1111_1110, 0b0010_1000]);
        let (packet, bits) = parse_packet(BitSlice::new(&slice, 0));
        assert_eq!(packet.version, 6);
        assert_eq!(packet.type_id, 4);
        assert_eq!(packet.payload, Payload::Literal(2021));

        // handle case single nibble
        let slice = Vec::from([0b1101_0001, 0b0100_0000]);
        let (packet, bits) = parse_packet(BitSlice::new(&slice, 0));
        assert_eq!(packet.version, 6);
        assert_eq!(packet.type_id, 4);
        assert_eq!(packet.payload, Payload::Literal(10));

        let slice = Vec::from([0b1111_0100, 0b0111_1000]);
        let (packet, bits) = parse_packet(BitSlice::new(&slice, 2));
        assert_eq!(packet.version, 6);
        assert_eq!(packet.type_id, 4);
        assert_eq!(packet.payload, Payload::Literal(15));
    }

//    #[test]
//    fn test_length_0_packets() {
//        let slice = Vec::from([
//            0b0011_1000,
//            0b0000_0000,
//            0b0110_1111,
//            0b0100_0101,
//            0b0010_1001,
//            0b0001_0010,
//            0b0000_0000,
//        ]);
//        let (packet, bits) = parse_length_type_0_packets(BitSlice::new(&slice, 7));
//    }

    #[test]
    fn test_sum_packet_versions() {
        let stream: Vec<u8> = FromHex::from_hex("8A004A801A8002F478".trim()).unwrap();
        let bits = BitSlice::new(&stream,0);
        println!("{}", bits);
        let (packet,_) = parse_packet(bits);
        let sum = sum_version_numbers(&packet);
        println!("{:?}", packet);
        assert_eq!(sum, 16);
    }

}
