/*
 * AXON IPC Demo — Receiver Protection Domain (v2)
 * Handles protected procedure call from sender.
 * Copyright 2026 Edison Lepiten — AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

#define CHANNEL_TO_SENDER 1

extern int32_t axon_receiver_logic(int32_t x);

void init(void) {
    microkit_dbg_puts("AXON IPC Demo: Receiver PD ACTIVE\n");
}

void notified(microkit_channel ch) { (void)ch; }

microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    int32_t val = (int32_t)microkit_mr_get(0);
    microkit_dbg_puts("Receiver: got value via PPC = ");
    char buf[4];
    buf[0] = '0' + (val / 10);
    buf[1] = '0' + (val % 10);
    buf[2] = '\n'; buf[3] = 0;
    microkit_dbg_puts(buf);

    int32_t result = axon_receiver_logic(val);
    microkit_mr_set(0, (seL4_Word)result);
    (void)ch;
    return microkit_msginfo_new(0, 1);
}
