use log::{error, trace, warn};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::vec::Vec;

/**
 * A trait representing a addressable memory region (ROM or RAM) in the Gameboy.
 */
pub trait MemoryChunk {
  fn write_u8(&mut self, address: u16, value: u8);
  fn read_u8(&self, address: u16) -> u8;
}

impl dyn MemoryChunk {
  pub fn write_u16(&mut self, address: u16, value: u16) {
    let lower = value & 0xFF;
    let upper = value >> 8;
    self.write_u8(address + 1, upper as u8);
    self.write_u8(address, lower as u8);
  }
  pub fn read_u16(&mut self, address: u16) -> u16 {
    let upper = self.read_u8(address + 1);
    let lower = self.read_u8(address);
    let result = ((upper as u16) << 8) + (lower as u16);
    result
  }
}

pub type MemoryPtr = dyn MemoryChunk + 'static;

/**
 * Read only chunk of memory loaded as bytes
 */
pub struct RomChunk {
  pub bytes: Vec<u8>,
}

impl MemoryChunk for RomChunk {
  fn write_u8(&mut self, address: u16, _: u8) {
    warn!("tried to write to {:x} in RomChunk", address);
  }
  fn read_u8(&self, address: u16) -> u8 {
    //trace!("read from {:x} in RomChunk", address);
    self.bytes[address as usize]
  }
}

impl RomChunk {
  pub fn from_file(path: &str) -> io::Result<RomChunk> {
    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(RomChunk { bytes: buffer })
  }
}

/**
 * RAM read/write memory as bytes
 */
pub struct RamChunk {
  pub bytes: Vec<u8>,
}

impl MemoryChunk for RamChunk {
  fn write_u8(&mut self, address: u16, v: u8) {
    //trace!("write {} to {:x} in RamChunk", v, address);
    self.bytes[address as usize] = v;
  }
  fn read_u8(&self, address: u16) -> u8 {
    //trace!("read from {:x} in RamChunk", address);
    self.bytes[address as usize]
  }
}

impl RamChunk {
  pub fn new(size: usize) -> RamChunk {
    RamChunk {
      bytes: vec![0; size],
    }
  }
}

pub struct GameboyState {
  boot: RomChunk,
  cart: RomChunk,
  cart_ram: RamChunk,
  vram: RamChunk,
  work_ram_one: RamChunk,
  work_ram_two: RamChunk,
  high_ram: RamChunk,
  boot_enabled: bool,
  pub a: bool,
  pub b: bool,
  pub start: bool,
  pub select: bool,
  pub left: bool,
  pub right: bool,
  pub up: bool,
  pub down: bool,
  gamepad_high: bool,
}

impl GameboyState {
  pub fn new(boot: RomChunk, cart: RomChunk) -> GameboyState {
    GameboyState {
      boot: boot,
      cart: cart,
      cart_ram: RamChunk::new(0x2000),
      vram: RamChunk::new(0x2000),
      work_ram_one: RamChunk::new(0x1000),
      work_ram_two: RamChunk::new(0x1000),
      high_ram: RamChunk::new(0x200),
      boot_enabled: true,
      a: false,
      b: false,
      start: false,
      select: false,
      left: false,
      right: false,
      up: false,
      down: false,
      gamepad_high: false,
    }
  }
}

impl MemoryChunk for GameboyState {
  fn write_u8(&mut self, address: u16, val: u8) {
    trace!("write {:x} to {:x}", val, address);

    if address < 0x8000 {
      error!("Illegal write to ROM {}", address);
    } else if address < 0xA000 {
      self.vram.write_u8(address - 0x8000, val)
    } else if address < 0xC000 {
      self.cart_ram.write_u8(address - 0xA000, val)
    } else if address < 0xD000 {
      self.work_ram_one.write_u8(address - 0xC000, val)
    } else if address < 0xE000 {
      self.work_ram_two.write_u8(address - 0xD000, val)
    } else if self.boot_enabled && address == 0xFF50 {
      // Writing a 1 to this register disables the boot rom
      self.boot_enabled = false;
    } else if address < 0xFE00 {
      // TODO: mirror ram, do I need?
      unimplemented!();
    } else {
      if address == 0xFF00 {
        if val & (1 << 4) != 0 {
          self.gamepad_high = false;
        } else if val & (1 << 5) != 0 {
          self.gamepad_high = true;
        }
      } else {
        self.high_ram.write_u8(address - 0xFE00, val)
      }
    }
  }
  fn read_u8(&self, address: u16) -> u8 {
    trace!("read {:x}", address);

    if address < 0x8000 {
      if self.boot_enabled && address <= 0x100 {
        return self.boot.read_u8(address);
      }
      self.cart.read_u8(address)
    } else if address < 0xA000 {
      self.vram.read_u8(address - 0x8000)
    } else if address < 0xC000 {
      self.cart_ram.read_u8(address - 0xA000)
    } else if address < 0xD000 {
      self.work_ram_one.read_u8(address - 0xC000)
    } else if address < 0xE000 {
      self.work_ram_two.read_u8(address - 0xD000)
    } else if address < 0xFE00 {
      // TODO: mirror ram, do I need?
      unimplemented!();
    } else {
      if address == 0xFF00 {
        let mut pad_state = 0;
        if self.gamepad_high {
          //A, B, Select, Start
          if !self.a {
            pad_state |= 1;
          }
          if !self.b {
            pad_state |= 2;
          }
          if !self.select {
            pad_state |= 4;
          }
          if !self.start {
            pad_state |= 8;
          }
        } else {
          //Right left up down
          if !self.left {
            pad_state |= 1;
          }
          if !self.right {
            pad_state |= 2;
          }
          if !self.up {
            pad_state |= 4;
          }
          if !self.down {
            pad_state |= 8;
          }
        }
        pad_state
      } else {
        self.high_ram.read_u8(address - 0xFE00)
      }
    }
  }
}
