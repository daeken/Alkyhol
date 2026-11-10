//! PUSHA/POPA decoder gap adapter. Only wholly admitted frames execute.
//! Partial stack faults and wrapped frames spanning address zero fail explicitly.
use crate::memory::{Memory, READ, WRITE};
pub(crate) unsafe extern "C" fn execute(
    memory: *mut Memory,
    registers: *mut u64,
    operation: u64,
) -> u64 {
    let memory = unsafe { &mut *memory };
    let regs = unsafe { std::slice::from_raw_parts_mut(registers, 91) };
    let pop = operation & 1 != 0;
    let width = if operation & 2 != 0 { 2 } else { 4 };
    let length = 8 * width;
    let sp = regs[4] as u32;
    let address = if pop {
        sp
    } else {
        sp.wrapping_sub(length as u32)
    };
    if memory
        .check(address as u64, length, if pop { READ } else { WRITE })
        .is_err()
    {
        memory.cpu_fault.get_or_insert(
            "PUSHA/POPA frame outside admitted storage; partial stack faults unimplemented",
        );
        return 0;
    }
    let order = [7, 6, 5, 4, 3, 2, 1, 0];
    if pop {
        let data = memory.read(address as u64, length, READ).unwrap();
        let mut values = [0u64; 8];
        for (i, reg) in order.into_iter().enumerate() {
            if reg == 4 {
                continue;
            }
            let value = if width == 2 {
                u16::from_le_bytes(data[i * 2..i * 2 + 2].try_into().unwrap()) as u64
            } else {
                u32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().unwrap()) as u64
            };
            values[reg] = if width == 2 {
                (regs[reg] & 0xffff0000) | value
            } else {
                value
            };
        }
        for reg in [0, 1, 2, 3, 5, 6, 7] {
            regs[reg] = values[reg];
        }
        regs[4] = sp.wrapping_add(length as u32) as u64;
    } else {
        let mut data = [0u8; 32];
        for (i, reg) in order.into_iter().enumerate() {
            let bytes = (regs[reg] as u32).to_le_bytes();
            data[i * width..(i + 1) * width].copy_from_slice(&bytes[..width]);
        }
        memory.write(address as u64, &data[..length]).unwrap();
        regs[4] = address as u64;
    }
    1
}
