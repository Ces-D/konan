use rongta::SupportedDriver;

const VENDOR_ID: u16 = 0x0FE6;
const PRODUCT_ID: u16 = 0x811E;

pub fn driver() -> SupportedDriver {
    SupportedDriver::Usb(VENDOR_ID, PRODUCT_ID)
}
