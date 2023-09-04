#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

#include "../harnesses/utils.h"

float a[A_ROWS * A_COLS] __attribute__((section(".dram0.data")));
float b[B_ROWS * B_COLS] __attribute__((section(".dram0.data")));
float c[A_ROWS * B_COLS] __attribute__((section(".dram0.data")));

float c_spec[A_ROWS * B_COLS] __attribute__((section(".dram0.data")));

void clang(float* input_A, float* input_B, float* input_C);
void isaria(float* input_A, float* input_B, float* input_C);

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

  int time = 0;

  /* clang(a, b, c); */
  /* clang(a, b, c); */
  /* clang(a, b, c); */
  /* zero_matrix(c, A_ROWS, B_COLS); */

  start_cycle_timing;
  clang(a, b, c);
  stop_cycle_timing;
  time = get_time();
  print_matrix(c, A_ROWS, B_COLS);
  printf("clang.vec : %d cycles\n", time);
  fprintf(file,
          "%s,%d,%d,%d,%d,%d,%d\n",
          "clang.vec",
          A_ROWS,
          A_COLS,
          B_ROWS,
          B_COLS,
          time,
          1
          );

  zero_matrix(c, A_ROWS, B_COLS);
  start_cycle_timing;
  isaria(a, b, c);
  stop_cycle_timing;
  time = get_time();
  print_matrix(c, A_ROWS, B_COLS);
  zero_matrix(c, A_ROWS, B_COLS);
  printf("isaria : %d cycles\n", time);
  fprintf(file,
          "%s,%d,%d,%d,%d,%d,%d\n",
          "isaria",
          A_ROWS,
          A_COLS,
          B_ROWS,
          B_COLS,
          time,
          1
          );

  return 0;
}
