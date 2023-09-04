#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

// Naive hard-coded size
void __attribute__((noinline)) clang(float * __restrict a, float * __restrict b, float * __restrict c) {
  for (int y = 0; y < A_ROWS; y++) {
    for (int x = 0; x < B_COLS; x++) {
      c[B_COLS * y + x] = 0;
      for (int k = 0; k < A_COLS; k++) {
        c[B_COLS * y + x] += a[A_COLS * y + k] * b[B_COLS * k + x];
      }
    }
  }
}

