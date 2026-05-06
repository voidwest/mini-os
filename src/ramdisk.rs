use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

/// size of the ramdisk in bytes (1 MiB).
const RAMDISK_SIZE: usize = 1024 * 1024;

lazy_static! {
    static ref RAMDISK: Mutex<Ramdisk> = Mutex::new(Ramdisk::new(RAMDISK_SIZE));
}

/// in-memory block device backed by a `Vec<u8>`.
pub struct Ramdisk {
    data: Vec<u8>,
}

impl Ramdisk {
    fn new(size: usize) -> Self {
        Ramdisk { data: alloc::vec![0u8; size] }
    }

    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        if offset >= self.data.len() {
            return Err("offset out of bounds");
        }
        let available = self.data.len() - offset;
        let len = buf.len().min(available);
        buf[..len].copy_from_slice(&self.data[offset..offset + len]);
        Ok(len)
    }

    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize, &'static str> {
        if offset >= self.data.len() {
            return Err("offset out of bounds");
        }
        let available = self.data.len() - offset;
        let len = buf.len().min(available);
        self.data[offset..offset + len].copy_from_slice(&buf[..len]);
        Ok(len)
    }
}

/// read from the global ramdisk into a buffer. returns bytes read.
pub fn read(offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    RAMDISK.lock().read(offset, buf)
}

/// write a buffer to the global ramdisk. returns bytes written.
pub fn write(offset: usize, buf: &[u8]) -> Result<usize, &'static str> {
    RAMDISK.lock().write(offset, buf)
}

/// return the size of the ramdisk in bytes.
pub fn size() -> usize {
    RAMDISK.lock().data.len()
}
