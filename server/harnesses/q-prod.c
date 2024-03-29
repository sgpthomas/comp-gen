#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

/* #include "quaternion_product.h" */
#include "utils.h"

float a_q[4] __attribute__((section(".dram0.data")));
float a_t[3] __attribute__((section(".dram0.data")));
float b_q[4] __attribute__((section(".dram0.data")));
float b_t[3] __attribute__((section(".dram0.data")));
float r_q[4] __attribute__((section(".dram0.data")));
float r_t[4] __attribute__((section(".dram0.data")));

float r_q_spec[4] __attribute__((section(".dram0.data")));
float r_t_spec[3] __attribute__((section(".dram0.data")));

// Diospyros kernel
void kernel(float * a_q, float * a_t, float * b_q, float * b_t, /* inputs */
            float * r_q, float * r_t);                          /* outputs */


__attribute__((always_inline)) void naive_cross_product(float *lhs, float *rhs, float *result) {
  result[0] = lhs[1] * rhs[2] - lhs[2] * rhs[1];
  result[1] = lhs[2] * rhs[0] - lhs[0] * rhs[2];
  result[2] = lhs[0] * rhs[1] - lhs[1] * rhs[0];
}

/*
  Computes the point product
*/
__attribute__((always_inline)) void naive_point_product(float *q, float *p, float *result) {
  float qvec[3] = {q[0], q[1], q[2]};
  float uv[3];
  naive_cross_product(qvec, p, uv);

  for (int i = 0; i < 3; i++) {
    uv[i] = uv[i] * 2;
  }
  float qxuv[3];
  naive_cross_product(qvec, uv, qxuv);

  for (int i = 0; i < 3; i++) {
    result[i] = p[i] + q[3]*uv[i] + qxuv[i];
  }
}

void naive_quaternion_product(float * a_q, float * a_t,
                              float * b_q, float * b_t,
                              float * r_q, float * r_t) {
  r_q[3] = a_q[3]*b_q[3] - a_q[0]*b_q[0] - a_q[1]*b_q[1] - a_q[2]*b_q[2];
  r_q[0] = a_q[3]*b_q[0] + a_q[0]*b_q[3] + a_q[1]*b_q[2] - a_q[2]*b_q[1];
  r_q[1] = a_q[3]*b_q[1] + a_q[1]*b_q[3] + a_q[2]*b_q[0] - a_q[0]*b_q[2];
  r_q[2] = a_q[3]*b_q[2] + a_q[2]*b_q[3] + a_q[0]*b_q[1] - a_q[1]*b_q[0];

  naive_point_product(a_q, b_t, r_t);
  for (int i = 0; i < 3; i++) {
    r_t[i] += a_t[i];
  }
}

int main(int argc, char **argv) {

  FILE *file = fopen(OUTFILE, "w");
  if (file == NULL) file = stdout;

  fprintf(file, "kernel,cycles,correct\n");

  init_rand(10);

  create_random_mat(a_q, 4, 1);
  create_random_mat(a_t, 3, 1);
  create_random_mat(b_q, 4, 1);
  create_random_mat(b_t, 3, 1);

  zero_matrix(r_q, 4, 1);
  zero_matrix(r_t, 3, 1);
  zero_matrix(r_q_spec, 4, 1);
  zero_matrix(r_t_spec, 3, 1);

  printf("Inputs\n");
  print_matrix(a_q, 4, 1);
  print_matrix(a_t, 3, 1);
  print_matrix(b_q, 4, 1);
  print_matrix(b_t, 3, 1);

  // Spec
  printf("Spec\n");
  naive_quaternion_product(a_q, a_t, b_q, b_t, r_q_spec, r_t_spec);
  print_matrix(r_q_spec, 4, 1);
  print_matrix(r_t_spec, 3, 1);

  int time = 0;

  printf("Results\n");

  // Naive
  start_cycle_timing;
  naive_quaternion_product(a_q, a_t, b_q, b_t, r_q, r_t);
  stop_cycle_timing;
  time = get_time();
  print_matrix(r_q, 4, 1);
  print_matrix(r_t, 3, 1);
  output_check(r_q, r_q_spec, 4, 1);
  output_check(r_t, r_t_spec, 3, 1);
  zero_matrix(r_q, 4, 1);
  zero_matrix(r_t, 3, 1);
  printf("Naive hard size : %d cycles\n", time);
  /* fprintf(file, "%s,%d\n","Naive hard size",time); */

  /* // Eigen */
  /* Eigen::Map<Eigen::Vector4f> aq_(a_q, 4, 1); */
  /* Eigen::Map<Eigen::Vector3f> at_(a_t, 3, 1); */
  /* SE3T a_ = {aq_, at_}; */

  /* Eigen::Map<Eigen::Vector4f> bq_(b_q, 4, 1); */
  /* Eigen::Map<Eigen::Vector3f> bt_(b_t, 3, 1); */
  /* SE3T b_ = {bq_, bt_}; */

  /* // Only time the kernel call, not the datatype conversions */
  /* start_cycle_timing; */
  /* SE3T c_ = quaternionProduct(a_, b_); */
  /* stop_cycle_timing; */
  /* time = get_time(); */

  /* memcpy(r_q, c_.quaternion.data(), sizeof(float) * 4); */
  /* memcpy(r_t, c_.translation.data(), sizeof(float) * 3); */
  /* print_matrix(r_q, 4, 1); */
  /* print_matrix(r_t, 3, 1); */
  /* output_check(r_q, r_q_spec, 4, 1); */
  /* output_check(r_t, r_t_spec, 3, 1); */
  /* zero_matrix(r_q, 4, 1); */
  /* zero_matrix(r_t, 3, 1); */
  /* printf("Eigen : %d cycles\n", time); */
  /* fprintf(file, "%s,%d\n","Eigen",time); */

  // Original Diospryos

  // Diospyros-like ruleset

  // Comp-gen
  start_cycle_timing;
  kernel(a_q, a_t, b_q, b_t, r_q, r_t);
  stop_cycle_timing;
  time = get_time();
  print_matrix(r_q, 4, 1);
  print_matrix(r_t, 3, 1);
  bool correct1 = output_check(r_q, r_q_spec, 4, 1);
  bool correct2 = output_check(r_t, r_t_spec, 3, 1);
  zero_matrix(r_q, 4, 1);
  zero_matrix(r_t, 3, 1);
  printf("compgen : %d cycles\n", time);
  fprintf(file, "%s,%d,%d\n", "compgen", time, correct1 & correct2);

  return 0;
}
