/* Bounded PE32 address-hole allocation. mmap is deliberately used WITHOUT
 * MAP_FIXED: racing host allocations must never be replaced.
 * The SDK supplies the real Mach region layouts and port types. */
#include <mach/mach.h>
#include <mach/mach_vm.h>
#include <stdint.h>
#include <stdio.h>
#include <errno.h>
#include <sys/mman.h>

int alkyhol_pe32_allocate_hole(uint64_t minimum, uint64_t end, uint64_t size,
                             uint64_t alignment, uint64_t *output) {
#if defined(__aarch64__) || defined(__arm64__)
    size=(size+16383)&~UINT64_C(16383);
    if (alignment<16384) alignment=16384;
#endif
    if (!output || minimum < 65536 || end >
#if defined(__aarch64__) || defined(__arm64__)
        UINT64_C(0x8100000000)
#else
        UINT64_C(0x100000000)
#endif
        ||
        minimum >= end || !size || size % 4096 || alignment < 4096 ||
        (alignment & (alignment - 1)) || alignment > UINT64_C(0x100000000))
        return KERN_INVALID_ARGUMENT;
    mach_vm_address_t candidate = (minimum + alignment - 1) & ~(alignment - 1);
    for (unsigned attempts = 0; attempts < 100000; ++attempts) {
        if (candidate >= end || size > end - candidate) return KERN_NO_SPACE;
        mach_vm_address_t region = candidate;
        mach_vm_size_t length = 0;
        vm_region_basic_info_data_64_t info;
        mach_msg_type_number_t count = VM_REGION_BASIC_INFO_COUNT_64;
        mach_port_t object = MACH_PORT_NULL;
        kern_return_t status = mach_vm_region(mach_task_self(), &region, &length,
            VM_REGION_BASIC_INFO_64, (vm_region_info_t)&info, &count, &object);
        if (object != MACH_PORT_NULL) mach_port_deallocate(mach_task_self(), object);
        if (status != KERN_SUCCESS && status != KERN_INVALID_ADDRESS) return status;
        uint64_t gap_end = status == KERN_INVALID_ADDRESS || region > end ? end : region;
        if (candidate < gap_end && size <= gap_end - candidate) {
            // mmap without MAP_FIXED is also safe against a racing allocation.
            // Rosetta's MAP_32BIT path admits low mappings that the ordinary
            // mach_vm_allocate entry point can reject as INVALID_ADDRESS.
#if defined(__aarch64__) || defined(__arm64__)
            mach_vm_address_t fixed = candidate;
            // FIXED without OVERWRITE never replaces a racing allocation.
            status = mach_vm_allocate(mach_task_self(), &fixed, size, VM_FLAGS_FIXED);
            if (status != KERN_SUCCESS) { candidate += alignment; continue; }
            void *mapped = (void *)(uintptr_t)fixed;
#else
            void *mapped = mmap((void *)(uintptr_t)candidate, size, PROT_READ | PROT_WRITE,
                               MAP_PRIVATE | MAP_ANON
#if defined(__x86_64__)
                               | 0x8000 /* MAP_32BIT */
#endif
                               , -1, 0);
            if (mapped == MAP_FAILED) { fprintf(stderr,"PE32 mmap failed: address=%llx size=%llx errno=%d\n",(unsigned long long)candidate,(unsigned long long)size,errno); return KERN_RESOURCE_SHORTAGE; }
#endif
            uint64_t chosen = (uint64_t)(uintptr_t)mapped;
            if (chosen >= minimum && chosen < end && size <= end - chosen && !(chosen & (alignment - 1))) {
                *output = chosen;
                return KERN_SUCCESS;
            }
            munmap(mapped, size);
            candidate += alignment;
            continue;
        }
        if (status == KERN_INVALID_ADDRESS) return KERN_NO_SPACE;
        if (!length || region > UINT64_MAX - length) return KERN_FAILURE;
        uint64_t next = region + length;
        if (next <= candidate || next > UINT64_MAX - (alignment - 1)) return KERN_FAILURE;
        candidate = (next + alignment - 1) & ~(alignment - 1);
    }
    return KERN_ABORTED; // Explicit bounded retry exhaustion, never success/OOM inference.
}

uint64_t alkyhol_pe32_resident_bytes(void) {
    mach_task_basic_info_data_t info;
    mach_msg_type_number_t count=MACH_TASK_BASIC_INFO_COUNT;
    if(task_info(mach_task_self(),MACH_TASK_BASIC_INFO,(task_info_t)&info,&count)!=KERN_SUCCESS)
        return UINT64_MAX;
    return info.resident_size;
}
