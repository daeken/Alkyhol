/* Exercise the production transition gates with simultaneous i386 entries. */
#include <assert.h>
#include <mach/mach.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

extern int alkyhol_pe32_allocate_hole(uint64_t, uint64_t, uint64_t, uint64_t, uint64_t *);
extern int i386_get_ldt(int, uint64_t *, int);
extern int i386_set_ldt(int, const uint64_t *, int);
extern uint64_t alkyhol_enter_pe32(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t);
extern const uint8_t probe_guest_start[], probe_guest_end[];
extern const uint8_t alkyhol_pe32_gate_start[], alkyhol_pe32_gate_end[];
extern const uint8_t alkyhol_pe32_dispatch_slot[];

static _Thread_local uint16_t host_fs;
static pthread_mutex_t dispatch_lock = PTHREAD_MUTEX_INITIALIZER;
static uint32_t calls;

static uint16_t current_fs(void) {
    uint16_t fs;
    __asm__ volatile("movw %%fs,%0" : "=r"(fs));
    return fs;
}

static uint64_t dispatch(uint32_t id, const uint32_t *args) {
    (void)args;
    assert(current_fs() == host_fs && id == 77);
    assert(pthread_mutex_lock(&dispatch_lock) == 0);
    calls++;
    assert(pthread_mutex_unlock(&dispatch_lock) == 0);
    return 0x78563412;
}

struct worker {
    uint32_t entry, stack, fs, marker;
    uint64_t result;
};

static void *run(void *argument) {
    struct worker *worker = argument;
    host_fs = current_fs();
    worker->result = alkyhol_enter_pe32(worker->entry, worker->stack, 0x1007, 0x100f, worker->fs);
    assert(current_fs() == host_fs);
    return NULL;
}

static void write_u32(uint8_t *address, uint32_t value) {
    memcpy(address, &value, sizeof(value));
}

static void install_gates(uint8_t *code, uint32_t base) {
    uint16_t cs;
    __asm__ volatile("movw %%cs,%0" : "=r"(cs));

    // Far jump to the real host CS, capture guest ESP, then return via R13.
    const uint8_t return64[] = {0x89, 0xe2, 0x4c, 0x89, 0xe4, 0x41, 0xff, 0xe5};
    uint8_t *return_gate = code + 0x100;
    return_gate[0] = 0xff;
    return_gate[1] = 0x2d;
    write_u32(return_gate + 2, base + 0x108);
    write_u32(return_gate + 8, base + 0x110);
    memcpy(return_gate + 12, &cs, sizeof(cs));
    memcpy(return_gate + 16, return64, sizeof(return64));

    // Push an import ID and far jump into the copied production gateway.
    uint8_t *import = code + 0x200;
    import[0] = 0x68;
    write_u32(import + 1, 77);
    import[5] = 0xff;
    import[6] = 0x2d;
    write_u32(import + 7, base + 0x210);
    write_u32(import + 16, base + 0x240);
    memcpy(import + 20, &cs, sizeof(cs));

    const uintptr_t template = (uintptr_t)alkyhol_pe32_gate_start;
    const size_t size = (uintptr_t)alkyhol_pe32_gate_end - template;
    const size_t slot = (uintptr_t)alkyhol_pe32_dispatch_slot - template;
    assert(size <= 192 && slot + sizeof(uintptr_t) <= size);
    memcpy(code + 0x240, alkyhol_pe32_gate_start, size);
    const uintptr_t callback = (uintptr_t)dispatch;
    memcpy(code + 0x240 + slot, &callback, sizeof(callback));
}

int main(int argc, char **argv) {
    if (argc > 2 || (argc == 2 && strcmp(argv[1], "--cold") != 0)) {
        fprintf(stderr, "usage: %s [--cold]\n", argv[0]);
        return 2;
    }
    // A runtime that serializes these two entries cannot pass the guest rendezvous.
    alarm(20);
    uint64_t address = 0;
    assert(alkyhol_pe32_allocate_hole(0x10000, 0x7fff0000, 0x14000, 0x10000, &address) == KERN_SUCCESS);
    uint8_t *memory = (void *)(uintptr_t)address;
    const uint32_t base = (uint32_t)address;
    const size_t guest_size = (uintptr_t)probe_guest_end - (uintptr_t)probe_guest_start;
    assert(guest_size <= 0x100);
    memcpy(memory, probe_guest_start, guest_size);
    install_gates(memory, base);
    assert(mprotect(memory, 0x4000, PROT_READ | PROT_EXEC) == 0);

    uint64_t entries[4] = {0};
    const int count = i386_get_ldt(512, entries, 4);
    assert(count >= 0 && count <= 8192);
    for (int i = 0; i < 4; i++)
        assert(count <= 512 + i || entries[i] == 0);

    const uint64_t flat[] = {0x00cffa000000ffffULL, 0x00cff2000000ffffULL};
    for (int i = 0; i < 2; i++)
        assert(i386_set_ldt(512 + i, &flat[i], 1) == 512 + i);

    struct worker workers[2];
    pthread_t threads[2];
    for (uint32_t i = 0; i < 2; i++) {
        const uint32_t teb = base + 0x4000 * (1 + i);
        const uint32_t stack = base + 0x4000 * (4 + i) - 16;
        const uint64_t descriptor = 0x0040f20000000fffULL
            | ((uint64_t)(teb & 0xffff) << 16)
            | ((uint64_t)((teb >> 16) & 0xff) << 32)
            | ((uint64_t)(teb >> 24) << 56);
        assert(i386_set_ldt(514 + (int)i, &descriptor, 1) == 514 + (int)i);

        const uint32_t marker = 0x12345678U + i;
        write_u32(memory + (teb - base) + 0x18, marker);
        const uint32_t frame[] = {base + 0x100, base + 0x5000, marker, base + 0x200};
        memcpy(memory + (stack - base), frame, sizeof(frame));
        workers[i] = (struct worker){base, stack, 0x1017U + 8U * i, marker, 0};
    }
    // Run these instruction paths once before testing simultaneous entry.
    // The optional cold case reproduces the independent Rosetta failure study.
    if (argc == 1) {
        write_u32(memory + 0x5000, 1);
        run(&workers[0]);
        assert((uint32_t)workers[0].result == workers[0].marker);
        assert((workers[0].result >> 32) == workers[0].stack + 16);
        assert(calls == 4096);
        write_u32(memory + 0x5000, 0);
        calls = 0;
    }
    for (unsigned i = 0; i < 2; i++)
        assert(pthread_create(&threads[i], NULL, run, &workers[i]) == 0);
    for (unsigned i = 0; i < 2; i++) {
        assert(pthread_join(threads[i], NULL) == 0);
        assert((uint32_t)workers[i].result == workers[i].marker);
        assert((workers[i].result >> 32) == workers[i].stack + 16);
    }

    uint32_t arrived;
    memcpy(&arrived, memory + 0x5000, sizeof(arrived));
    assert(arrived == 2 && calls == 8192);
    const uint64_t empty = 0;
    for (int i = 0; i < 4; i++)
        assert(i386_set_ldt(512 + i, &empty, 1) == 512 + i);
    assert(munmap(memory, 0x14000) == 0);
    puts("Two concurrent i386 guests preserved FS across 8,192 production gate calls");
}
