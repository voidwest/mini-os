use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
};

lazy_static! {
    static ref PHYS_MEM_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);
}

/// store the physical memory offset for later use (page-table walks).
fn set_phys_mem_offset(offset: VirtAddr) {
    *PHYS_MEM_OFFSET.lock() = Some(offset);
}

/// init paging: construct an offset page table from the active level-4 table.
///
/// # Safety
/// the caller must ensure physical memory is identity-mapped at the given
/// `physical_memory_offset`.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    set_phys_mem_offset(physical_memory_offset);
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// translate a virtual address to physical by walking page tables.
///
/// # Safety
/// the caller must ensure physical memory is mapped at the given
/// `physical_memory_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()];

    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages unsupported"),
        };
    }
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// frame allocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// frame allocator from the bootloader memory map (sequential).
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
    /// create a new `BootInfoFrameAllocator` from the bootloader memory map.
    ///
    /// # Safety
    /// the memory map must be valid and all usable frames must be free.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map, next: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// map a page to the vga buffer at `0xb8000`. demo of page-table manipulation.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map to failed").flush();
}

/// mark a page as user-accessible (ring 3). handles 4KiB and huge pages.
///
/// # Safety
/// the caller must ensure `addr` points to kernel-controlled memory that is
/// safe to expose to user mode.
pub unsafe fn mark_page_user(addr: VirtAddr) {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::PageTableFlags;

    let offset = PHYS_MEM_OFFSET.lock().expect("memory::init not called");

    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()];
    let mut frame = level_4_table_frame;

    for (level, &index) in table_indexes.iter().enumerate() {
        let virt = offset + frame.start_address().as_u64();
        let table_ptr: *mut PageTable = virt.as_mut_ptr();
        let table = unsafe { &mut *table_ptr };
        let entry = &mut table[index];

        // last level (p1) — always a 4KiB page.
        if level == 3 {
            entry.set_flags(entry.flags() | PageTableFlags::USER_ACCESSIBLE);
            return;
        }

        // huge page at any upper level — set flag directly on the huge entry.
        if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            entry.set_flags(entry.flags() | PageTableFlags::USER_ACCESSIBLE);
            return;
        }

        // walk to the next page-table level.
        frame = match entry.frame() {
            Ok(f) => f,
            Err(_) => return,
        };
    }
}
