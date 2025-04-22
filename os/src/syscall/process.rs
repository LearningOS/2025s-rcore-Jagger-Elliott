//! Process management syscalls

use crate::{
    mm::{translated_byte_buffer, PageTable, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count, mmap, munmap, suspend_current_and_run_next
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let ptr = ts as *mut u8;
    let len = core::mem::size_of::<TimeVal>();
    let buffers = translated_byte_buffer(current_user_token(), ptr, len);

    let bytes = unsafe { core::slice::from_raw_parts(&timeval as *const _ as *const u8, len) };
    let mut written = 0;
    for buf in buffers {
        buf.copy_from_slice(&bytes[written..written + buf.len()]);
        written += buf.len();
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let page_table = PageTable::from_token(current_user_token());
    match trace_request {
        0 => {
            if let Some(pte) = page_table.translate(VirtAddr::from(id).floor()) {
                if !pte.user_accessible() || !pte.readable() || !pte.is_valid() {
                    return -1isize;
                }
            } else {
                return -1isize;
            }
            let id = id as *const u8;
            let buffers = translated_byte_buffer(current_user_token(), id, 1);
            return buffers[0][0] as isize;
        }
        1 => {
            if let Some(pte) = page_table.translate(VirtAddr::from(id).floor()) {
                if !pte.user_accessible() || !pte.writable() || !pte.is_valid() {
                    return -1isize;
                }
            } else {
                return -1isize;
            }
            let id = id as *const u8;
            let mut buffers = translated_byte_buffer(current_user_token(), id, 1);
            buffers[0][0] = (data & 0xFF) as u8;
            return 0;
        }
        2 => get_syscall_count(id),

        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    
    return mmap(start, len, port);
    
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {

    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    return munmap(start, len);

}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
