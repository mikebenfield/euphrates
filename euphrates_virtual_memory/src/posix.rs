use std::mem::size_of;
use std::ptr;

use rand::{FromEntropy, Rng, XorShiftRng};

use super::*;
use traits::{Protect, UnsafeMemory, VirtualMemory};

const PAGE_SIZE: usize = 4096;

unsafe fn check_error<T: Eq>(val: T, err_val: T, s: &str) -> T {
    use std::mem::transmute;
    if val == err_val {
        libc::perror(transmute(s.as_ptr()));
        panic!("C error");
    }
    val
}

pub struct PhysicalMemory<T> {
    fd: i32,
    mem: LogicalMemory<T>,
}

impl<T: Copy> UnsafeMemory<T> for PhysicalMemory<T> {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *const T {
        self.mem.as_ptr()
    }

    #[inline(always)]
    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, index: usize) -> T {
        self.mem.get_unchecked(index)
    }

    #[inline(always)]
    unsafe fn set_unchecked(&mut self, index: usize, val: T) {
        self.mem.set_unchecked(index, val);
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline(always)]
    fn pages(&self) -> usize {
        self.mem.pages()
    }
}

impl<T> Drop for PhysicalMemory<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            check_error(libc::close(self.fd), -1, "drop physical memory\0");
        }
    }
}

pub struct LogicalMemory<T> {
    ptr: *mut T,
    len: usize,
}

impl<T> Protect for LogicalMemory<T> {
    #[inline]
    unsafe fn set_protection(&mut self, write: bool, execute: bool) {
        use std::mem::transmute;
        let mut prot = libc::PROT_READ;
        if write {
            prot |= libc::PROT_WRITE;
        }
        if execute {
            prot |= libc::PROT_EXEC;
        }
        check_error(
            libc::mprotect(transmute(self.ptr), self.len * size_of::<T>(), prot),
            -1,
            "set_protection\0",
        );
    }
}

impl<T: Copy> UnsafeMemory<T> for LogicalMemory<T> {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *const T {
        self.ptr
    }

    #[inline(always)]
    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, index: usize) -> T {
        *self.ptr.offset(index as isize)
    }

    unsafe fn set_unchecked(&mut self, index: usize, val: T) {
        *self.ptr.offset(index as isize) = val;
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn pages(&self) -> usize {
        self.len * size_of::<T>() / PAGE_SIZE
    }
}

impl<T> Drop for LogicalMemory<T> {
    #[inline]
    fn drop(&mut self) {
        use std::mem::transmute;
        unsafe {
            check_error(
                libc::munmap(transmute(self.ptr), self.len * size_of::<T>()),
                -1,
                "drop logical memory\0",
            );
        }
    }
}

#[inline]
fn allocate_logical<T: Copy>(
    len: usize,
    write: bool,
    execute: bool,
    flags: i32,
    fd: i32,
) -> LogicalMemory<T> {
    use std::mem::transmute;

    let mut prot: i32 = libc::PROT_READ;
    if write {
        prot |= libc::PROT_WRITE;
    }
    if execute {
        prot |= libc::PROT_EXEC;
    }
    let len2 = len * size_of::<T>();
    let ptr = unsafe {
        transmute(check_error(
            libc::mmap(ptr::null_mut(), len2, prot, flags, fd, 0),
            libc::MAP_FAILED,
            "allocate_logical\0",
        ))
    };
    LogicalMemory { ptr, len }
}

pub struct Main;

impl<T: Copy> VirtualMemory<T> for Main {
    const PAGE_SIZE: usize = PAGE_SIZE;

    type PhysicalMemory = PhysicalMemory<T>;

    type LogicalMemory = LogicalMemory<T>;

    unsafe fn allocate_physical(len: usize, write: bool, execute: bool) -> PhysicalMemory<T> {
        let mut oflag = libc::O_CREAT | libc::O_EXCL;
        if write {
            oflag |= libc::O_RDWR;
        } else {
            oflag |= libc::O_RDONLY;
        }
        let mut rand = XorShiftRng::from_entropy();
        let name: [i8; 9] = [
            47,
            rand.gen(),
            rand.gen(),
            rand.gen(),
            rand.gen(),
            rand.gen(),
            rand.gen(),
            rand.gen(),
            0,
        ];
        let fd = check_error(
            libc::shm_open(
                name.as_ptr(),
                oflag,
                (libc::S_IRUSR | libc::S_IWUSR) as libc::c_uint,
            ),
            -1,
            "allocate_physical\0",
        );
        let len2 = len * size_of::<T>();
        check_error(libc::shm_unlink(name.as_ptr()), -1, "allocate_physical\0");
        check_error(libc::ftruncate(fd, len2 as i64), -1, "allocate_physical\0");
        let logical = allocate_logical(len, write, execute, libc::MAP_SHARED, fd);
        PhysicalMemory { fd, mem: logical }
    }

    #[inline]
    unsafe fn allocate_logical(len: usize, write: bool, execute: bool) -> Self::LogicalMemory {
        allocate_logical(len, write, execute, libc::MAP_SHARED | libc::MAP_ANON, -1)
    }

    #[inline]
    unsafe fn map(
        logical: &mut Self::LogicalMemory,
        logical_offset: usize,
        physical: &Self::PhysicalMemory,
        physical_offset: usize,
        len: usize,
        write: bool,
        execute: bool,
    ) {
        use std::mem::transmute;
        let sz = size_of::<T>();
        let logical_ptr = logical.ptr.offset(logical_offset as isize);
        let mut prot = libc::PROT_READ;
        if write {
            prot |= libc::PROT_WRITE;
        }
        if execute {
            prot |= libc::PROT_READ;
        }
        check_error(
            libc::mmap(
                transmute(logical_ptr),
                len * sz,
                prot,
                libc::MAP_SHARED | libc::MAP_FIXED,
                physical.fd,
                (physical_offset * sz) as i64,
            ),
            libc::MAP_FAILED,
            "map\0",
        );
    }
}
