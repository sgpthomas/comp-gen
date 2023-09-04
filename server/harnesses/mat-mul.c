#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <Eigen/Core>
#include <Eigen/Dense>

#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

#include "utils.h"

/* #define OUTFILE "mat-mul.csv" */

float a[A_ROWS * A_COLS] __attribute__((section(".dram0.data")));
float b[B_ROWS * B_COLS] __attribute__((section(".dram0.data")));
float c[A_ROWS * B_COLS] __attribute__((section(".dram0.data")));

float c_spec[A_ROWS * B_COLS] __attribute__((section(".dram0.data")));

/* extern "C" { */
/*   // Expert kernel */
/*   void matrix_multiply_2x3_3x3_expert(float* c, float* a, float* b); */
/*   // Nature kernel */
/*   /\* void matmmltf(const float32_t *x, int M, int N, const float32_t *y, *\/ */
/*   /\*   int P, float32_t *z); *\/ */
/* } */

// Diospyros kernel
void kernel(float* input_A, float* input_B, float* input_C);


// Naive
void __attribute__((noinline)) naive_matrix_multiply(float * __restrict a,
                                                     float * __restrict b,
                                                     float * __restrict c,
                                                     int row1, int col1, int col2) {
  for (int y = 0; y < row1; y++) {
    for (int x = 0; x < col2; x++) {
      c[col2 * y + x] = 0;
      for (int k = 0; k < col1; k++) {
        c[col2 * y + x] += a[col1 * y + k] * b[col2 * k + x];
      }
    }
  }
}

int main(int argc, char **argv) {

  FILE *file = fopen(OUTFILE, "w");
  if (file == NULL) file = stdout;
  fprintf(file, "kernel,A_ROWS,A_COLS,B_ROWS,B_COLS,cycles,correct\n");

  init_rand(10);

  create_random_mat(a, A_ROWS, A_COLS);
  create_random_mat(b, B_ROWS, B_COLS);
  zero_matrix(c, A_ROWS, B_COLS);

  print_matrix(a, A_ROWS, A_COLS);
  print_matrix(b, B_ROWS, B_COLS);

  // Run naive once to warm cache
  naive_matrix_multiply(a, b, c_spec,  A_ROWS, A_COLS, B_COLS);
  zero_matrix(c, A_ROWS, B_COLS);

  int time = 0;

  // Comp Gen
  start_cycle_timing;
  kernel(a, b, c);
  stop_cycle_timing;
  time = get_time();
  print_matrix(c, A_ROWS, B_COLS);
  bool correct = output_check(c, c_spec, A_ROWS, B_COLS);
  zero_matrix(c, A_ROWS, B_COLS);
  printf("compgen : %d cycles\n", time);
  fprintf(file,
          "%s,%d,%d,%d,%d,%d,%d\n",
          "compgen",
          A_ROWS,
          A_COLS,
          B_ROWS,
          B_COLS,
          time,
          correct
          );

  return 0;
}
