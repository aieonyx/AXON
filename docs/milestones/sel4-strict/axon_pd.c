/*
 * AXON seL4 Protection Domain — Milestone 2
 * Calls AXON compiled code, prints result.
 * Copyright 2026 Edison Lepiten — AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

/* AXON compiled function — linked from axon_prog.o */
extern int32_t axon_main(void);

/* Convert int to decimal string in buf, return length */
static int int_to_str(int32_t n, char *buf) {
    if (n == 0) { buf[0] = '0'; buf[1] = 0; return 1; }
    char tmp[12]; int i = 0, len = 0;
    if (n < 0) { buf[len++] = '-'; n = -n; }
    while (n > 0) { tmp[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) { buf[len++] = tmp[--i]; }
    buf[len] = 0;
    return len;
}

void init(void) {
    microkit_dbg_puts("AXON seL4-strict domain: ACTIVE\n");
    microkit_dbg_puts("Running AXON program...\n");

    int32_t result = axon_main();

    char buf[32];
    microkit_dbg_puts("axon_main() returned: ");
    int_to_str(result, buf);
    microkit_dbg_puts(buf);
    microkit_dbg_puts("\n");

    if (result == 42) {
        microkit_dbg_puts("AXON seL4 MILESTONE 2: PASSED — 21 + 21 = 42\n");
    } else {
        microkit_dbg_puts("AXON seL4 MILESTONE 2: UNEXPECTED RESULT\n");
    }
}

void notified(microkit_channel ch) {
    (void)ch;
}
