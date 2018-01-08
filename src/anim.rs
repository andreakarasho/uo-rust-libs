//! Methods for reading animated characters out of anim.mul/anim.idx
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use color::{Color, Color16, Color32};
use mul_reader::MulReader;
use std::fs::{File};
use std::io::{Result, Error, ErrorKind, Cursor, SeekFrom, Seek, Read};
use std::path::Path;
use image::{Frames, Frame, Rgba, RgbaImage};
use num_rational::Ratio;

const PALETTE_SIZE: usize = 256;
const IMAGE_COMPLETE: u32 = 0x7FFF7FFF;

const X_MASK: u32 = 0x007FF000;
const Y_MASK: u32 = 0x007FF000; //FIXME: wrong

pub struct Row {
    pub header: u32,
    pub image_data: Vec<u8>
}

impl Row {
    pub fn x_offset(&self) -> i16 {
        ((self.header & X_MASK) >> 6) as i16  //FIXME
    }

    pub fn y_offset(&self) -> i16 {
        ((self.header & Y_MASK) >> 6) as i16 //FIXME: wrong
    }
}

pub struct AnimFrame {
    pub image_centre_x: i16,
    pub image_centre_y: i16,
    pub width: u16,
    pub height: u16,
    pub data: Vec<Row>
}

pub struct AnimGroup {
    pub palette: [Color16; 256],
    pub frame_count: u32,
    pub frames: Vec<AnimFrame>
}

impl AnimGroup {
    pub fn to_frames(&self) -> Frames {
        Frames::new(self.frames.iter().map(|anim_frame| {
            // TODO: Figure out what to do with image_centre_x and y, and sort out offsets
            let mut buffer = RgbaImage::new(anim_frame.width as u32, anim_frame.height as u32);
            for pixel in buffer.pixels_mut() {
                *pixel = Rgba([255, 255, 255, 255]);
            }
            Frame::from_parts(buffer, 0, 0, Ratio::from_integer(0))
        }).collect())
    }
}

pub struct AnimReader<T: Read + Seek> {
    mul_reader: MulReader<T>
}

fn read_frame<T: Read + Seek>(reader: &mut T) -> Result<AnimFrame> {
    let image_centre_x = try!(reader.read_i16::<LittleEndian>());
    let image_centre_y = try!(reader.read_i16::<LittleEndian>());
    let width = try!(reader.read_u16::<LittleEndian>());
    let height = try!(reader.read_u16::<LittleEndian>());

    let mut data = vec![];
    loop {
        let header = try!(reader.read_u32::<LittleEndian>());
        if header == IMAGE_COMPLETE {
            break;
        }
        let run_length = header & 0xFFF;
        let mut image_data = vec![];
        for _i in 0..run_length {
            image_data.push(try!(reader.read_u8()));
        }
        data.push(Row {
            header,
            image_data
        });
    }

    // Read data
    Ok(AnimFrame {
        image_centre_x: image_centre_x,
        image_centre_y: image_centre_y,
        width: width,
        height: height,
        data: data
    })
}

impl AnimReader<File> {

    pub fn new(index_path: &Path, mul_path: &Path) -> Result<AnimReader<File>> {
        let mul_reader = try!(MulReader::new(index_path, mul_path));
        Ok(AnimReader {
            mul_reader: mul_reader
        })
    }
}

impl <T: Read + Seek> AnimReader<T> {

    pub fn from_mul(reader: MulReader<T>) -> AnimReader<T> {
        AnimReader {
            mul_reader: reader
        }
    }

    pub fn read(&mut self, id: u32) -> Result<AnimGroup> {

        let raw = try!(self.mul_reader.read(id));
        let mut reader = Cursor::new(raw.data);
        // Read the palette
        let mut palette = [0; PALETTE_SIZE];
        for i in 0..PALETTE_SIZE {
            palette[i] = try!(reader.read_u16::<LittleEndian>());
        }

        let frame_count = try!(reader.read_u32::<LittleEndian>());
        let mut frame_offsets = vec![];
        for _ in 0..frame_count {
            frame_offsets.push(try!(reader.read_u32::<LittleEndian>()));
        }

        let mut frames = vec![];
        for offset in frame_offsets {
            try!(reader.seek(SeekFrom::Start((PALETTE_SIZE as u32 * 2 + offset) as u64)));
            frames.push(try!(read_frame(&mut reader)));
        }

        Ok(AnimGroup {
            palette: palette,
            frame_count: frame_count,
            frames: frames
        })
    }
}
