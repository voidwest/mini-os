// oh boy
use x86_64::{VirtAddr, structures::paging::PageTable};

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {}
