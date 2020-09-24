use crate::cpu::CPU;
use crate::gpu::{GpuStepState, GPU};
use crate::memory::{GameboyState, MemoryChunk};

/// Encapsulate the entire running state of the Gameboy
pub struct Machine {
  pub cpu: CPU,
  pub gpu: GPU,
  pub memory: GameboyState 
}

impl Machine {
  pub fn step(&mut self, screen_buffer: &mut [u8]) -> GpuStepState {
    self.cpu.step(&mut self.memory);
    self.gpu.step(&mut self.cpu, &mut self.memory, screen_buffer)
  }
}
