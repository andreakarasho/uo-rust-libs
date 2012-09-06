export get_writer;
export extract_muls;
export ByteBuffer;

struct ByteBuffer {
    bytes: ~[u8],
    mut length: uint,
    mut pos: uint
}

fn extract_muls(path: ~str, idx: ~str, mul: ~str, name: ~str) {
    let maybe_reader: option::Option<mul_reader::MulReader> = mul_reader::reader(path, idx, mul);

    if option::is_none(maybe_reader) {
        io::println("Error reading tiles");
        assert false;
    }

    let reader: mul_reader::MulReader = option::unwrap(maybe_reader);
    let mut index:uint = 0;
    while (reader.eof() != true) {
        let item: option::Option<mul_reader::MulRecord> = reader.read();
        if option::is_some(item) {
            let unwrapped: mul_reader::MulRecord = option::unwrap(item);
            slice_mul(unwrapped, #fmt("%s-%u", name, index))
        }
        index += 1;
    }
}

fn get_writer(path: ~str) -> io::Writer {
    
    let maybe_writer = io::file_writer(&path::Path(path), ~[io::Create, io::Truncate]);

    if result::is_err::<io::Writer, ~str>(maybe_writer) {
        io::println(#fmt("%s", result::unwrap_err(maybe_writer)));
        fail;
    }

    return result::unwrap(maybe_writer);
}

fn slice_mul(record: mul_reader::MulRecord, name: ~str) {
    let header: io::Writer = get_writer(#fmt("./output/%s.mulheader", name));
    let body: io::Writer = get_writer(#fmt("./output/%s.mulslice", name));
    io::u64_to_le_bytes(record.start as u64, 4u, |v| header.write(v));
    io::u64_to_le_bytes(record.length as u64, 4u, |v| header.write(v));
    io::u64_to_le_bytes(record.opt1 as u64, 2u, |v| header.write(v));
    io::u64_to_le_bytes(record.opt2 as u64, 2u, |v| header.write(v));
    body.write(record.data);
}

pure fn ByteBuffer(bytes: ~[u8]) -> ByteBuffer {
    return ByteBuffer {
        bytes: bytes,
        length: vec::len(bytes),
        pos: 0
    }
}

impl ByteBuffer {
    pure fn eof() -> bool {return self.pos == self.length;}
    fn read(number: uint) -> ~[u8] {
        io::println(#fmt("Read from %u to %u against %u", self.pos, number, self.length));
        assert (number + self.pos <= self.length);
        let return_data = vec::slice(self.bytes, self.pos, self.pos + number);
        self.pos += number;
        return return_data;
    }
    fn seek(pos: uint) {
        io::println(#fmt("Seek to %u", pos));
        assert pos >= 0 && pos <= self.length;
        self.pos = pos;
    }
}
