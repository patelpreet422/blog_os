/*
GDT
  The Global Descriptor Table (GDT) is a relict that was used for memory
  segmentation before paging became the de facto standard. It is still
  needed in 64-bit mode for various things such as kernel/user mode
  configuration or TSS loading.

  The GDT is a structure that contains the segments of the program.
  It was used on older architectures to isolate programs from each other,
  before paging became the standard. While segmentation is no longer supported
  in 64-bit mode, the GDT still exists. It is mostly used for two things:
  Switching between kernel space and user space, and loading a TSS structure.

IST and TSS
  The Interrupt Stack Table (IST) is part of an old legacy structure called
  Task State Segment (TSS). The TSS used to hold various information
  (e.g. processor register state) about a task in 32-bit mode and was for
  example used for hardware context switching. However, hardware context
  switching is no longer supported in 64-bit mode and the format of the TSS
  changed completely.

  On x86_64, the TSS no longer holds any task specific information at all.
  Instead, it holds two stack tables (the IST is one of them). The only
  common field between the 32-bit and 64-bit TSS is the pointer to the I/O
  port permissions bitmap.
*/

use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
  static ref TSS: TaskStateSegment = {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
      const STACK_SIZE: usize = 4096 * 5;
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

      let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
      let stack_end = stack_start + STACK_SIZE;
      stack_end
    };
    tss
  };
}

struct Selectors {
  code_selector: SegmentSelector,
  tss_selector: SegmentSelector,
}

lazy_static! {
  static ref GDT: (GlobalDescriptorTable, Selectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (
      gdt,
      Selectors {
        code_selector,
        tss_selector,
      },
    )
  };
}

pub fn init() {
  use x86_64::instructions::segmentation::set_cs;
  use x86_64::instructions::tables::load_tss;

  GDT.0.load();
  unsafe {
    set_cs(GDT.1.code_selector);
    load_tss(GDT.1.tss_selector);
  }
}
