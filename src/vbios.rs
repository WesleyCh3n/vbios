use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, Write};
use std::ops::Range;
use std::path::Path;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub struct VBiosBuilder<P: AsRef<Path> + Clone> {
    root: P,
    bins: Vec<(P, Vec<u8>)>,
}

impl<P: AsRef<Path> + Clone> VBiosBuilder<P> {
    pub fn new(root: P) -> Self {
        Self { root, bins: vec![] }
    }
    pub fn add_bin(mut self, path: P, flag: Vec<u8>) -> Self {
        self.bins.push((path, flag));
        self
    }
    pub fn build(&self) -> io::Result<VBios> {
        let mut buf = std::fs::read(&self.root)?;
        self.bins.clone().into_iter().for_each(|(p, mut f)| {
            f.extend_from_slice(&buf.len().to_le_bytes()[..4]);
            let bin = std::fs::read(&p).unwrap_or_else(|e| {
                panic!("couldn't read: {}. {}", p.as_ref().display(), e)
            });
            let mut encoder =
                ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&bin).unwrap();
            let com_buf = encoder.finish().unwrap();
            buf.extend_from_slice(&f);
            buf.extend_from_slice(&com_buf);
        });
        Ok(VBios::new(buf))
    }
}

#[derive(Default)]
pub struct VBios {
    buf: Vec<u8>,
}

impl<P: AsRef<Path>> From<P> for VBios {
    fn from(path: P) -> Self {
        Self {
            buf: std::fs::read(&path).unwrap_or_else(|e| {
                panic!("couldn't read: {}. {}", path.as_ref().display(), e)
            }),
        }
    }
}

impl VBios {
    fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }
    pub fn size(&self) -> usize {
        self.buf.len()
    }
    pub fn write_all<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        output_file.write_all(&self.buf)?;
        Ok(())
    }
    pub fn find_flag(&self, flag: Vec<u8>) -> usize {
        let f_len = flag.len();
        self.buf
            .windows(f_len)
            .enumerate()
            .position(|(i, window)| {
                window == flag
                    && i == u32::from_le_bytes(
                        // u32 because we use 4 bytes to store size
                        (&self.buf[i + f_len..i + f_len + 4])
                            .try_into()
                            .unwrap(),
                    ) as usize
            })
            .unwrap_or_else(|| panic!("couldn't find flag: {:?}", flag))
    }
    pub fn export_bin<P: AsRef<Path>>(
        &self,
        path: P,
        range: Range<usize>,
    ) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut deflater = ZlibDecoder::new(&self.buf[range]);
        let mut dec_buf = Vec::new();
        deflater.read_to_end(&mut dec_buf).unwrap();
        Ok(file.write_all(&dec_buf)?)
    }
}
