//! Process management syscalls
use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::mm::{translate_to_phys_addr, VirtAddr};
use crate::timer::get_time_us;
use crate::task :: {syscall_get,syscall_mmap,syscall_unmap};
use crate::config::PAGE_SIZE;
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
    let token = current_user_token();
    let phys_addr:usize = translate_to_phys_addr(token, ts as usize);
    let us = get_time_us();
    unsafe {
        *(phys_addr as *mut TimeVal) = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let phys_addr:usize = translate_to_phys_addr(token, id);
    if phys_addr == 0 {
        return -1;
    }
    let phys_ptr = phys_addr as *mut u8;
    match trace_request {
        0 => {
            let value = unsafe {
                *phys_ptr as isize 
            };
            value
        },
        1 => {
            
            unsafe {
                *phys_ptr = data as u8;
            };
            0
        },
        2 => {
            syscall_get(id) as isize
        },
        _ => {
            -1
        },
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // 检查 prot 是否只有前三位有效
    if port & !0b111 != 0 {
        return -1; // prot 包含无效位，其他位必须为 0
    }
    if port & 0b111 == 0{
        return -1;
    }
    if len == 0 || start % PAGE_SIZE != 0 {
        return -1; // 非法的 `len` 或 `start` 地址不对齐
    }
    syscall_mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1; // 非法的 start 地址
    }
    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start+len).into();
    if  !start_va.aligned() || !end_va.aligned(){
        return -1;
    }
    // 检查参数合法性
    if len == 0 || start % PAGE_SIZE != 0 {
        return -1; // 非法的 `len` 或 `start` 地址不对齐
    }
    syscall_unmap(start, len)
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
