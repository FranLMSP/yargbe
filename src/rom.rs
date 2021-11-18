#[cfg(not(test))]
use std::fs::File;
#[cfg(not(test))]
use std::io::Read;

use crate::bus::{
    BANK_ZERO,
    BANK_SWITCHABLE,
    EXTERNAL_RAM,
};

pub const CARTRIDGE_TYPE_ADDRESS: u16 = 0x0147;
pub const CGB_FLAG_ADDRESS: u16 = 0x0143;
pub const SGB_FLAG_ADDRESS: u16 = 0x0146;
pub const RAM_SIZE_ADDRESS: u16 = 0x0149;
pub const ROM_SIZE_ADDRESS: u16 = 0x0148;
pub const DESTINATION_CODE_ADDRESS: u16 = 0x014A;
pub const HEADER_CHECKSUM_ADDRESS: u16 = 0x014D;

#[cfg(not(test))]
fn header_checksum(data: &Vec<u8>) -> bool {
    if data.len() < HEADER_CHECKSUM_ADDRESS as usize {
        return false;
    }

    let mut checksum: u8 = 0;
    let mut index: u16 = 0x0134;
    while index < HEADER_CHECKSUM_ADDRESS {
        checksum = checksum.wrapping_sub(data[index as usize]).wrapping_sub(1);
        index += 1;
    }
    checksum == data[HEADER_CHECKSUM_ADDRESS as usize]
}

#[cfg(test)]
pub fn load_rom(_filename: &str) -> std::io::Result<Box<dyn ROM>> {
    Ok(Box::new(NoMBC::new(Vec::new(), ROMInfo {
        mbc: MBC::NoMBC,
        publisher: "".to_string(),
        title: "".to_string(),
        cgb_only: false,
        sgb_features: false,
        has_ram: false,
        has_battery: false,
        has_timer: false,
        ram_banks: 0,
        rom_banks: 2,
        region: Region::NonJapanese,
    })))
}

#[cfg(not(test))]
pub fn load_rom(filename: &str) -> std::io::Result<Box<dyn ROM>> {
    let mut file = File::open(filename)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    if !header_checksum(&data) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Header checksum failed. Is this a Gameboy ROM?"));
    }

    let info = ROMInfo::from_bytes(&data);

    Ok(match info.mbc {
        MBC::NoMBC => Box::new(NoMBC::new(data, info)),
        MBC::MBC1 => Box::new(MBC1::new(data, info)),
        MBC::MBC2 => Box::new(MBC2::new(data, info)),
        MBC::MBC3 => Box::new(MBC3::new(data, info)),
        _ => unimplemented!(),
    })
}

#[derive(Debug, Copy, Clone)]
enum MBC {
    NoMBC,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    HuC1,
    HuC3,
    MMM01,
    PocketCamera,
    BandaiTIMA5,
}

#[derive(Debug)]
enum Region {
    Japanese,
    NonJapanese,
}

#[derive(Debug, PartialEq)]
enum BankingMode {
    Simple,
    Advanced,
}

#[derive(Debug)]
pub struct ROMInfo {
    mbc: MBC,
    publisher: String,
    title: String,
    cgb_only: bool,
    sgb_features: bool,
    has_ram: bool,
    has_battery: bool,
    has_timer: bool,
    ram_banks: u8,
    rom_banks: u16,
    region: Region,
}

impl ROMInfo {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let rom_type = bytes[CARTRIDGE_TYPE_ADDRESS as usize];
        Self {
            mbc: match rom_type {
                0x00 => MBC::NoMBC,
                0x01 => MBC::MBC1,
                0x02 => MBC::MBC1,
                0x03 => MBC::MBC1,
                0x05 => MBC::MBC2,
                0x06 => MBC::MBC2,
                0x08 => MBC::NoMBC,
                0x09 => MBC::NoMBC,
                0x0B => MBC::MMM01,
                0x0C => MBC::MMM01,
                0x0F => MBC::MBC3,
                0x10 => MBC::MBC3,
                0x11 => MBC::MBC3,
                0x12 => MBC::MBC3,
                0x13 => MBC::MBC3,
                0x19 => MBC::MBC5,
                0x1A => MBC::MBC5,
                0x1B => MBC::MBC5,
                0x1C => MBC::MBC5,
                0x1D => MBC::MBC5,
                0x1E => MBC::MBC5,
                0x20 => MBC::MBC6,
                0x22 => MBC::MBC7,
                0xFC => MBC::PocketCamera,
                0xFD => MBC::BandaiTIMA5,
                0xFE => MBC::HuC3,
                0xFF => MBC::HuC1,
                _ => unreachable!(),
            },
            region: match bytes[DESTINATION_CODE_ADDRESS as usize] {
                0x00 => Region::Japanese,
                _ => Region::NonJapanese,
            },
            publisher: "".to_string(), // TODO: Extract publisher
            title: "".to_string(), // TODO: Extract the game title
            cgb_only: bytes[CGB_FLAG_ADDRESS as usize] == 0xC0,
            sgb_features: bytes[SGB_FLAG_ADDRESS as usize] == 0x03,
            has_ram: match rom_type {
                0x02 | 0x03 | 0x08 | 0x09 | 0x0C | 0x0D | 0x10 | 0x12 |
                0x13 | 0x1A | 0x1B | 0x1D | 0x1E | 0x22 | 0xFF => true,
                _ => false,
            },
            has_battery: match rom_type {
                0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 |
                0x13 | 0x1B | 0x1E | 0x22 | 0xFF => true,
                _ => false,
            },
            has_timer: match rom_type {
                0x0F | 0x10 => true,
                _ => false,
            },
            ram_banks: match bytes[RAM_SIZE_ADDRESS as usize] {
                0x00 | 0x01 => 0,
                0x02 => 1,
                0x03 => 4,
                0x04 => 16,
                0x05 => 8,
                _ => unreachable!(),
            },
            rom_banks: match bytes[ROM_SIZE_ADDRESS as usize] {
                0x00 => 2,
                0x01 => 4,
                0x02 => 8,
                0x03 => 16,
                0x04 => 32,
                0x05 => 64,
                0x06 => 128,
                0x07 => 256,
                0x08 => 512,
                0x52 => 72,
                0x53 => 80,
                0x54 => 96,
                _ => unreachable!(),
            },
        }
    }

    pub fn ram_size(&self) -> usize {
        0x2000 * self.ram_banks as usize
    }
}

pub trait ROM {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

pub struct NoMBC {
    data: Vec<u8>,
    info: ROMInfo,
}

impl NoMBC {
    pub fn new(data: Vec<u8>, info: ROMInfo) -> Self {
        let rom = Self {
            data,
            info,
        };
        println!("MBC {:?}", rom.info.mbc);
        println!("Region {:?}", rom.info.region);

        rom
    }
}

impl ROM for NoMBC {
    fn read(&self, address: u16) -> u8 {
        match self.data.get(address as usize) {
            Some(byte) => *byte,
            None => 0xFF,
        }
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}

pub struct MBC1 {
    data: Vec<u8>,
    info: ROMInfo,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enable: bool,
    banking_mode: BankingMode,
}

impl MBC1 {
    fn new(data: Vec<u8>, info: ROMInfo) -> Self {
        println!("MBC {:?}", info.mbc);
        println!("Region {:?}", info.region);
        println!("Has RAM {}", info.has_ram);
        println!("ROM banks {}", info.rom_banks);
        println!("RAM banks {}", info.ram_banks);
        let ram = Vec::with_capacity(info.ram_size() as usize);
        Self {
            data,
            info,
            ram,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            banking_mode: BankingMode::Simple,
        }
    }

    fn switch_rom_bank(&mut self, bank: u16) {
        self.rom_bank = bank;
        if (self.rom_bank & 0b11111) == 0 {
            self.rom_bank += 1;
        }
        if self.rom_bank > self.info.rom_banks.saturating_sub(1) {
            self.rom_bank = self.info.rom_banks.saturating_sub(1);
        }
    }

    fn get_bank_zero_address(&self, address: u16) -> usize {
        match self.banking_mode {
            BankingMode::Simple => (address & 0x3FFF) as usize,
            BankingMode::Advanced => {
                ((self.ram_bank as usize) << 5) * ((address & 0x3FFF) as usize)
                // ((self.ram_bank as usize) << 19) | (address & 0x3FFF) as usize
            },
        }
    }

    fn get_bank_switchable_address(&self, address: u16) -> usize {
        let rom_bank = ((self.ram_bank as u16) << 5) | (self.rom_bank & 0b11111);
        ((rom_bank as usize) << 14) | (address & 0x3FFF) as usize
    }

    fn get_ram_address(&self, address: u16) -> usize {
        let bank = match self.banking_mode {
            BankingMode::Simple => 0,
            BankingMode::Advanced => self.ram_bank,
        };
        ((bank as usize) << 11) + ((address as usize) & 0x1FFF)
    }
}

impl ROM for MBC1 {
    fn read(&self, address: u16) -> u8 {
        if BANK_ZERO.contains(&address) {
            return match self.data.get(self.get_bank_zero_address(address)) {
                Some(byte) => *byte,
                None => 0xFF,
            };
        } else if BANK_SWITCHABLE.contains(&address) {
            return match self.data.get(self.get_bank_switchable_address(address)) {
                Some(byte) => *byte,
                None => 0xFF,
            };
        } else if EXTERNAL_RAM.contains(&address) {
            if !self.info.has_ram && !self.ram_enable {
                return 0xFF;
            }
            return match self.ram.get(self.get_ram_address(address)) {
                Some(data) => *data,
                None => 0xFF,
            };
        }
        unreachable!("ROM read: Address {} not valid", address);
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= 0x1FFF { // RAM enable register
            if !self.info.has_ram {
                return;
            }
            self.ram_enable = match data & 0x0F {
                0x0A => true,
                _ => false,
            };
            return;
        } else if address >= 0x2000 && address <= 0x3FFF { // ROM bank number register
            self.switch_rom_bank(data as u16 & 0b00011111);
        } else if address >= 0x4000 && address <= 0x5FFF { // ROM and RAM bank number register
            self.ram_bank = data & 0b11;
        } else if address >= 0x6000 && address <= 0x7FFF { // Banking mode select
            self.banking_mode = match (data & 1) == 0 {
                true => BankingMode::Simple,
                false => BankingMode::Advanced,
            }
        } else if EXTERNAL_RAM.contains(&address) {
            if !self.ram_enable || !self.info.has_ram {
                return;
            }
            let address = self.get_ram_address(address);
            if let Some(elem) = self.ram.get_mut(address) {
                *elem = data;
            }
        }
    }
}

pub struct MBC2 {
    data: Vec<u8>,
    info: ROMInfo,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enable: bool,
}

impl MBC2 {
    fn new(data: Vec<u8>, info: ROMInfo) -> Self {
        println!("MBC {:?}", info.mbc);
        println!("Region {:?}", info.region);
        println!("Has RAM {}", info.has_ram);
        println!("ROM banks {}", info.rom_banks);
        println!("RAM banks {}", info.ram_banks);
        let ram = Vec::with_capacity(info.ram_size() as usize);
        Self {
            data,
            info,
            ram,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
        }
    }

    fn switch_rom_bank(&mut self, bank: u16) {
        self.rom_bank = bank;
        if self.rom_bank > self.info.rom_banks.saturating_sub(1) {
            self.rom_bank = self.info.rom_banks.saturating_sub(1);
        } else if self.rom_bank == 0 {
            self.rom_bank = 1;
        }
    }
}

impl ROM for MBC2 {
    fn read(&self, address: u16) -> u8 {
        if BANK_ZERO.contains(&address) {
            return self.data[address as usize];
        } else if BANK_SWITCHABLE.contains(&address) {
            return self.data[((self.rom_bank as usize * 0x4000) + (address as usize & 0x3FFF)) as usize];
        } else if address >= 0xA000 && address <= 0xA1FF {
            if !self.info.has_ram || !self.ram_enable {
                return 0xFF;
            }
            return match self.ram.get((address - EXTERNAL_RAM.min().unwrap() + (0x2000 * self.ram_bank as u16)) as usize) {
                Some(data) => *data,
                None => 0xFF,
            };
        } else if address >= 0xA200 && address <= 0xBFFF {
            return self.read(0xA000 + (address % 0x0200));
        }
        return 0xFF;
    }

    fn write(&mut self, address: u16, data: u8) {
        if BANK_SWITCHABLE.contains(&address) {
            if address.to_be_bytes()[0] & 1 == 0 {
                match data {
                    0x0A => self.ram_enable = true,
                    _ => self.ram_enable = false,
                }
            } else {
                self.switch_rom_bank(data as u16);
            }
        }
    }
}

pub struct MBC3 {
    data: Vec<u8>,
    info: ROMInfo,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_timer_enable: bool,
}

impl MBC3 {
    fn new(data: Vec<u8>, info: ROMInfo) -> Self {
        println!("MBC {:?}", info.mbc);
        println!("Region {:?}", info.region);
        println!("Has RAM {}", info.has_ram);
        println!("ROM banks {}", info.rom_banks);
        println!("RAM banks {}", info.ram_banks);
        let ram = Vec::with_capacity(info.ram_size() as usize);
        Self {
            data,
            info,
            ram,
            rom_bank: 1,
            ram_bank: 0,
            ram_timer_enable: false,
        }
    }

    fn switch_rom_bank(&mut self, bank: u16) {
        self.rom_bank = bank;
        if self.rom_bank > self.info.rom_banks.saturating_sub(1) {
            self.rom_bank = self.info.rom_banks.saturating_sub(1);
        } else if self.rom_bank == 0 {
            self.rom_bank = 1;
        }
    }
}

impl ROM for MBC3 {
    fn read(&self, address: u16) -> u8 {
        if BANK_ZERO.contains(&address) {
            return self.data[address as usize];
        } else if BANK_SWITCHABLE.contains(&address) {
            return self.data[((self.rom_bank as usize * 0x4000) + (address as usize & 0x3FFF)) as usize];
        } else if EXTERNAL_RAM.contains(&address) {
            if !self.info.has_ram || !self.ram_timer_enable {
                return 0xFF;
            }
            return match self.ram.get((address - EXTERNAL_RAM.min().unwrap() + (0x2000 * self.ram_bank as u16)) as usize) {
                Some(data) => *data,
                None => 0xFF,
            };
        }
        return 0xFF;
    }

    fn write(&mut self, address: u16, data: u8) {
        if address >= 0xA000 && address <= 0xBFFF {
        } else if address <= 0x1FFF {
            match data {
                0x0A => self.ram_timer_enable = true,
                0x00 => self.ram_timer_enable = true,
                _ => {},
            }
        } else if address >= 0x2000 && address <= 0x3FFF {
            self.switch_rom_bank(data as u16);
        } else if address >= 0x4000 && address <= 0x5FFF {
            if data <= 0x03 {
                self.ram_bank = data;
            } else if data >= 0x08 && data <= 0x0C && self.info.has_timer {
            }
        } else if address >= 0x6000 && address <= 0x7FFF {
        }

    }
}
