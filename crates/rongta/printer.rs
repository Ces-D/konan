use anyhow::Result;
use escpos::{
    driver::{ConsoleDriver, NetworkDriver, UsbDriver},
    printer::Printer,
    utils::{JustifyMode, UnderlineMode},
};

pub enum AnyPrinter {
    Usb(Printer<UsbDriver>),
    Network(Printer<NetworkDriver>),
    Console(Printer<ConsoleDriver>),
}

macro_rules! delegate_printer_method {
    ($method:ident $(, $arg:ident : $ty:ty)*) => {
        pub fn $method(&mut self $(, $arg: $ty)*) -> Result<()> {
            match self {
                AnyPrinter::Usb(p) => { p.$method($($arg),*)?; },
                AnyPrinter::Network(p) => { p.$method($($arg),*)?; },
                AnyPrinter::Console(p)=>{ p.$method($($arg),*)?; }
            }
        Ok(())
        }
    };
}

impl AnyPrinter {
    delegate_printer_method!(feed);
    delegate_printer_method!(print);
    delegate_printer_method!(print_cut);
    delegate_printer_method!(write, text: &str);
    delegate_printer_method!(justify, mode: JustifyMode);
    delegate_printer_method!(bold, enabled: bool);
    delegate_printer_method!(underline, mode:UnderlineMode);
    delegate_printer_method!(size, width:u8, height:u8);
    delegate_printer_method!(reset_size);
}
