use core::cell::RefCell;
use core::hash::Hasher;
use embedded_storage::{ReadStorage, Storage};

use log::info;
use tickv::error_codes::ErrorCode;
use tickv::flash_controller::FlashController;
use tickv::{TicKV, MAIN_KEY};

const NVS_START_ADDRESS: u32 = 0x00009000; //from partition pable
const NVS_SIZE: usize = 0x00006000;

const REGION_SIZE: usize = 4096; //equal to sector size
const REGION_NUM: usize = NVS_SIZE as usize / REGION_SIZE;

#[inline]
fn get_hashed_key(unhashed_key: &[u8]) -> u64 {
    let mut hasher = wyhash::WyHash::with_seed(50);
    hasher.write(unhashed_key);
    hasher.finish()
}

struct FlashCtrl<T> {
    flash: RefCell<T>,
    buf: RefCell<[[u8; REGION_SIZE]; REGION_NUM]>,
}

impl<T> FlashCtrl<T>
where
    T: ReadStorage + Storage,
{
    fn new(t: T) -> Self {
        Self {
            flash: RefCell::new(t),
            buf: RefCell::new([[0xFF; REGION_SIZE]; REGION_NUM]),
        }
    }
}

impl<T> FlashController<REGION_SIZE> for FlashCtrl<T>
where
    T: ReadStorage + Storage,
{
    fn read_region(
        &self,
        region_number: usize,
        offset: usize,
        buf: &mut [u8; REGION_SIZE],
    ) -> Result<(), ErrorCode> {
        // TODO: Read the specified flash region
        self.flash
            .borrow_mut()
            .read(
                NVS_START_ADDRESS + (region_number * REGION_SIZE) as u32,
                &mut self.buf.borrow_mut()[region_number][..],
            )
            .map_err(|_| ErrorCode::ReadFail)?;
        // for (i, b) in buf.iter_mut().enumerate() {
        //     *b = self.buf.borrow()[region_number][offset + i]
        // }
        let len = buf.len().min(REGION_SIZE - offset);
        buf[..len].copy_from_slice(&self.buf.borrow()[region_number][offset..]);
        Ok(())
    }

    fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode> {
        // TODO: Write the data to the specified flash address
        self.flash
            .borrow_mut()
            .write(NVS_START_ADDRESS + address as u32, buf)
            .map_err(|_| ErrorCode::WriteFail)?;
        Ok(())
    }

    fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
        // TODO: Erase the specified flash region
        let buf = [0xFF; REGION_SIZE];
        self.write(region_number * REGION_SIZE, &buf)?;
        Ok(())
    }
}

pub fn flash_test() {
    let mut read_buf: [u8; REGION_SIZE] = [0; REGION_SIZE];
    let flash = esp_storage::FlashStorage::new();

    let tickv = TicKV::<FlashCtrl<esp_storage::FlashStorage>, REGION_SIZE>::new(
        FlashCtrl::new(flash),
        &mut read_buf,
        NVS_SIZE,
    );

    tickv.initialise(get_hashed_key(MAIN_KEY)).unwrap();

    let value: [u8; 31] = [7; 31];

    // Get the same key back
    let mut buf: [u8; 32] = [0; 32];
    tickv.get_key(get_hashed_key(b"ONE"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.invalidate_key(get_hashed_key(b"ONE")).unwrap();
    tickv.append_key(get_hashed_key(b"ONE"), &value).ok();
    tickv.get_key(get_hashed_key(b"ONE"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"TWO"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"THREE"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"FOUR"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"FIVE"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"SIX"), &mut buf).unwrap();
    info!("{:?}", buf);
    tickv.get_key(get_hashed_key(b"SEVEN"), &mut buf).unwrap();
    info!("{:?}", buf);
}
