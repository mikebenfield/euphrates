use std::ptr;

pub trait UnsafeMemory<T>
where
    T: Copy,
{
    unsafe fn as_ptr(&self) -> *const T;

    unsafe fn as_mut_ptr(&mut self) -> *mut T;

    #[inline]
    unsafe fn get(&self, index: usize) -> T {
        if index >= self.len() {
            index_fail(index, self.len())
        } else {
            self.get_unchecked(index)
        }
    }

    #[inline]
    unsafe fn set(&mut self, index: usize, val: T) {
        if index >= self.len() {
            index_fail(index, self.len())
        } else {
            self.set_unchecked(index, val)
        }
    }

    unsafe fn get_unchecked(&self, index: usize) -> T;

    unsafe fn set_unchecked(&mut self, index: usize, val: T);

    fn len(&self) -> usize;

    fn pages(&self) -> usize;
}

pub trait Protect {
    unsafe fn set_protection(&mut self, write: bool, execute: bool);
}

pub trait VirtualMemory<T: Copy> {
    const PAGE_SIZE: usize;

    type PhysicalMemory: UnsafeMemory<T>;

    type LogicalMemory: UnsafeMemory<T> + Protect;

    unsafe fn allocate_physical(len: usize, write: bool, execute: bool) -> Self::PhysicalMemory;

    unsafe fn allocate_logical(len: usize, write: bool, execute: bool) -> Self::LogicalMemory;

    unsafe fn dup(slice: &[T], execute: bool) -> Self::PhysicalMemory {
        // round up to a multiple of the page size
        let pre_len = (slice.len() + Self::PAGE_SIZE - 1) / Self::PAGE_SIZE;
        let len = pre_len * Self::PAGE_SIZE;
        let mut phys = Self::allocate_physical(len, true, execute);
        ptr::copy_nonoverlapping(slice.as_ptr(), phys.as_mut_ptr(), slice.len());
        phys
    }

    fn create_slice(phys: &Self::PhysicalMemory) -> Box<[T]> {
        use std::mem::forget;

        let mut vec = Vec::with_capacity(phys.len());
        let ptr = vec.as_mut_ptr();
        unsafe {
            forget(vec);
            ptr::copy_nonoverlapping(phys.as_ptr(), ptr, phys.len());
            Vec::from_raw_parts(ptr, phys.len(), phys.len()).into_boxed_slice()
        }
    }

    unsafe fn map(
        logical: &mut Self::LogicalMemory,
        logical_offset: usize,
        physical: &Self::PhysicalMemory,
        physical_offset: usize,
        len: usize,
        write: bool,
        execute: bool,
    );
}

#[inline]
fn index_fail(index: usize, len: usize) -> ! {
    panic!("index {} too large for memory of length {}", index, len);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn alloc_read<Vm: VirtualMemory<u8>>() {
        let mem = unsafe { Vm::allocate_physical(0x8000, false, false) };
        assert_eq!(mem.len(), 0x8000);
        let _x = unsafe { mem.get(104) };
        let _y = unsafe { mem.get(0x7FFF) };
    }

    pub fn read_write<Vm: VirtualMemory<u8>>() {
        let mut mem = unsafe { Vm::allocate_physical(0x8000, true, false) };
        unsafe {
            mem.set(0, 100);
            mem.set(2, 101);
            mem.set(0x7FFF, 102);
            assert_eq!(100, mem.get(0));
            assert_eq!(101, mem.get(2));
            assert_eq!(102, mem.get(0x7FFF));
        }
    }

    pub fn shared<Vm: VirtualMemory<u8>>() {
        unsafe {
            let mut log1 = Vm::allocate_logical(0x8000, true, false);
            let mut log2 = Vm::allocate_logical(0x8000, true, false);
            let mut phys = Vm::allocate_physical(0x8000, true, false);
            Vm::map(&mut log1, 0, &mut phys, 0, 0x8000, true, false);
            Vm::map(&mut log2, 0, &mut phys, 0, 0x8000, true, false);
            log1.set(0, 100);
            log1.set(1, 101);
            log1.set(0x7FFF, 102);
            log2.set(1000, 103);
            log2.set(0x7FFF, 104);
            assert_eq!(phys.get(0), 100);
            assert_eq!(phys.get(1), 101);
            assert_eq!(phys.get(1000), 103);
            assert_eq!(phys.get(0x7FFF), 104);
        }
    }

    pub fn shared2<Vm: VirtualMemory<u8>>() {
        unsafe {
            let mut log1 = Vm::allocate_logical(5 * 0x8000, true, false);
            let mut log2 = Vm::allocate_logical(5 * 0x8000, true, false);
            let mut log3 = Vm::allocate_logical(8 * 0x8000, false, false);
            let mut phys = Vm::allocate_physical(8 * 0x8000, true, false);
            Vm::map(&mut log1, 0x8000, &mut phys, 0, 4 * 0x8000, true, false);
            Vm::map(
                &mut log2,
                0x8000,
                &mut phys,
                4 * 0x8000,
                4 * 0x8000,
                true,
                false,
            );
            log1.set(0x8000, 0);
            log1.set(2 * 0x8000, 1);
            log1.set(3 * 0x8000, 2);
            log1.set(4 * 0x8000, 3);
            log2.set(0x8000, 4);
            log2.set(2 * 0x8000, 5);
            log2.set(3 * 0x8000, 6);
            log2.set(4 * 0x8000, 7);

            Vm::map(&mut log3, 0, &mut phys, 0, 8 * 0x8000, false, false);

            assert_eq!(log3.get(0), 0);
            assert_eq!(log3.get(0x8000), 1);
            assert_eq!(log3.get(2 * 0x8000), 2);
            assert_eq!(log3.get(3 * 0x8000), 3);
            assert_eq!(log3.get(4 * 0x8000), 4);
            assert_eq!(log3.get(5 * 0x8000), 5);
            assert_eq!(log3.get(6 * 0x8000), 6);
            assert_eq!(log3.get(7 * 0x8000), 7);
        }
    }

    pub fn shared_u32<Vm: VirtualMemory<u32>>() {
        unsafe {
            let mut log1 = Vm::allocate_logical(0x8000, true, false);
            let mut log2 = Vm::allocate_logical(0x8000, true, false);
            let mut phys = Vm::allocate_physical(0x8000, true, false);
            Vm::map(&mut log1, 0, &mut phys, 0, 0x8000, true, false);
            Vm::map(&mut log2, 0, &mut phys, 0, 0x8000, true, false);
            log1.set(0, 100);
            log1.set(1, 101);
            log1.set(0x7FFF, 102);
            log2.set(1000, 103);
            log2.set(0x7FFF, 104);
            assert_eq!(phys.get(0), 100);
            assert_eq!(phys.get(1), 101);
            assert_eq!(phys.get(1000), 103);
            assert_eq!(phys.get(0x7FFF), 104);
        }
    }
}
