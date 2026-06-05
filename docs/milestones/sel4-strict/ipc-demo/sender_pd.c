/*
 * AXON IPC Demo — Sender Protection Domain (v2)
 * Uses Microkit msginfo to transfer value via channel.
 * Copyright 2026 Edison Lepiten — AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

#define CHANNEL_TO_RECEIVER 1

extern int32_t axon_sender_logic(void);

static void print_int(int32_t n) {
    char buf[12]; int i = 0;
    if (n == 0) { microkit_dbg_puts("0"); return; }
    while (n > 0) { buf[i++] = '0' + (n % 10); n /= 10; }
    char out[12]; int len = 0;
    while (i > 0) { out[len++] = buf[--i]; }
    out[len] = 0;
    microkit_dbg_puts(out);
}

void init(void) {
    microkit_dbg_puts("AXON IPC Demo: Sender PD ACTIVE\n");
    int32_t val = axon_sender_logic();
    microkit_dbg_puts("Sender: axon_sender_logic() = ");
    print_int(val);
    microkit_dbg_puts("\n");

    /* Use protected procedure call to transfer value */
    microkit_msginfo info = microkit_msginfo_new(0, 1);
    microkit_mr_set(0, (seL4_Word)val);
    microkit_dbg_puts("Sender: calling receiver via PPC...\n");
    microkit_msginfo reply = microkit_ppcall(CHANNEL_TO_RECEIVER, info);

    int32_t result = (int32_t)microkit_mr_get(0);
    microkit_dbg_puts("Sender: received result = ");
    print_int(result);
    microkit_dbg_puts("\n");
    if (result == 42) {
        microkit_dbg_puts("AXON IPC DEMO: PASSED — sent 21, received 42\n");
    } else {
        microkit_dbg_puts("AXON IPC DEMO: UNEXPECTED RESULT\n");
    }
    (void)reply;
}

void notified(microkit_channel ch) { (void)ch; }
