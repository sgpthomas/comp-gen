#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

#include "utils.h"

float a[SIZE * SIZE] __attribute__((section(".dram0.data")));
float q[SIZE * SIZE] __attribute__((section(".dram0.data")));
float r[SIZE * SIZE] __attribute__((section(".dram0.data")));
float q_spec[SIZE * SIZE] __attribute__((section(".dram0.data")));
float r_spec[SIZE * SIZE] __attribute__((section(".dram0.data")));

// For Nature.
float scratch[SIZE] __attribute__((section(".dram0.data")));
float nat_v[(2*SIZE-SIZE+1)*(SIZE/2+SIZE)] __attribute__((section(".dram0.data")));
float nat_d[SIZE] __attribute__((section(".dram0.data")));
float nat_b[SIZE*SIZE] __attribute__((section(".dram0.data")));

// Compgen kernel
void kernel(float * A, float * Q, float * R);

float sgn(float v){
  return (v > 0) - (v < 0);
}

// Naive implementation
void naive_transpose(float *a, int n) {
  for (int i = 0; i < n; i++) {
    for (int j = i+1; j < n; j++) {
      float tmp = a[i*n + j];
      a[i*n + j] =  a[j*n + i];
      a[j*n + i] = tmp;
    }
  }
}

float naive_norm(float *x, int m) {
  float sum = 0;
  for (int i = 0; i < m; i++) {
    sum += pow(x[i], 2);
  }
  return sqrt(sum);
}

void naive_matrix_multiply(float *a, float *b, float *c, int row1, int col1, int col2) {
 for (int y = 0; y < row1; y++) {
   for (int x = 0; x < col2; x++) {
     c[col2 * y + x] = 0;
     for (int k = 0; k < col1; k++) {
       c[col2 * y + x] += a[col1 * y + k] * b[col2 * k + x];
     }
   }
 }
}

void naive_qr_decomp(float *A, float *Q, float *R, int n) {
  memcpy(R, A, sizeof(float) * n * n);

  // Build identity matrix of size n * n
  float *I = (float *)calloc(sizeof(float), n * n);
  for (int i = 0; i < n; i++) {
    for (int j = 0; j < n; j++) {
      I[i*n + j] = (i == j);
    }
  }

  // Householder
  for (int k = 0; k < n - 1; k++) {
    int m = n - k;

    float *x = (float *)calloc(sizeof(float), m);
    float *e = (float *)calloc(sizeof(float), m);
    for (int i = 0; i < m; i++) {
      int row = k + i;
      x[i] = R[row*n + k];
      e[i] = I[row*n + k];
    }

    float alpha = -sgn(x[0]) * naive_norm(x, m);

    float *u = (float *)calloc(sizeof(float), m);
    float *v = (float *)calloc(sizeof(float), m);
    for (int i = 0; i < m; i++) {
      u[i] = x[i] + alpha * e[i];
    }
    float norm_u = naive_norm(u, m);
    for (int i = 0; i < m; i++) {
      v[i] = u[i]/norm_u;
    }

    float *q_min = (float *)calloc(sizeof(float), m * m);
    for (int i = 0; i < m; i++) {
      for (int j = 0; j < m; j++) {
        float q_min_i = ((i == j) ? 1.0 : 0.0) - 2 * v[i] * v[j];
        q_min[i*m + j] = q_min_i;
      }
    }

    float *q_t = (float *)calloc(sizeof(float), n * n);
    for (int i = 0; i < n; i++) {
      for (int j = 0; j < n; j++) {
        float q_t_i;
        if ((i < k) || (j < k)) {
          q_t_i = (i == j) ? 1.0 : 0.0;
        } else {
          q_t_i =  q_min[(i - k)*m + (j - k)];
        }
        q_t[i*n + j] = q_t_i;
      }
    }

    if (k == 0) {
      memcpy(Q, q_t, sizeof(float) * n * n);     // Q = q_t
      naive_matrix_multiply(q_t, A, R, n, n, n); // R = q_t * A
    } else {
      float *res = (float *)calloc(sizeof(float), n * n);
      naive_matrix_multiply(q_t, Q, res, n, n, n); // R = q_t * A
      memcpy(Q, res, sizeof(float) * n * n);
      naive_matrix_multiply(q_t, R, res, n, n, n); // R = q_t * A
      memcpy(R, res, sizeof(float) * n * n);
    }
    free(x);
    free(e);
    free(u);
    free(v);
    free(q_min);
    free(q_t);
  }
  naive_transpose(Q, n);
}

// Naive with fixed size
void naive_fixed_transpose(float *a) {
  for (int i = 0; i < SIZE; i++) {
    for (int j = i+1; j < SIZE; j++) {
      float tmp = a[i*SIZE + j];
      a[i*SIZE + j] =  a[j*SIZE + i];
      a[j*SIZE + i] = tmp;
    }
  }
}

void naive_fixed_matrix_multiply(float *a, float *b, float *c) {
 for (int y = 0; y < SIZE; y++) {
   for (int x = 0; x < SIZE; x++) {
     c[SIZE * y + x] = 0;
     for (int k = 0; k < SIZE; k++) {
       c[SIZE * y + x] += a[SIZE * y + k] * b[SIZE * k + x];
     }
   }
 }
}

void naive_fixed_qr_decomp(float *A, float *Q, float *R) {
  memcpy(R, A, sizeof(float) * SIZE * SIZE);

  // Build identity matrix of size SIZE * SIZE
  float *I = (float *)calloc(sizeof(float), SIZE * SIZE);
  for (int i = 0; i < SIZE; i++) {
    for (int j = 0; j < SIZE; j++) {
      I[i*SIZE + j] = (i == j);
    }
  }

  // Householder
  for (int k = 0; k < SIZE - 1; k++) {
    int m = SIZE - k;

    float *x = (float *)calloc(sizeof(float), m);
    float *e = (float *)calloc(sizeof(float), m);
    for (int i = 0; i < m; i++) {
      int row = k + i;
      x[i] = R[row*SIZE + k];
      e[i] = I[row*SIZE + k];
    }

    float alpha = -sgn(x[0]) * naive_norm(x, m);

    float *u = (float *)calloc(sizeof(float), m);
    float *v = (float *)calloc(sizeof(float), m);
    for (int i = 0; i < m; i++) {
      u[i] = x[i] + alpha * e[i];
    }
    float norm_u = naive_norm(u, m);
    for (int i = 0; i < m; i++) {
      v[i] = u[i]/norm_u;
    }

    float *q_min = (float *)calloc(sizeof(float), m * m);
    for (int i = 0; i < m; i++) {
      for (int j = 0; j < m; j++) {
        float q_min_i = ((i == j) ? 1.0 : 0.0) - 2 * v[i] * v[j];
        q_min[i*m + j] = q_min_i;
      }
    }

    float *q_t = (float *)calloc(sizeof(float), SIZE * SIZE);
    for (int i = 0; i < SIZE; i++) {
      for (int j = 0; j < SIZE; j++) {
        float q_t_i;
        if ((i < k) || (j < k)) {
          q_t_i = (i == j) ? 1.0 : 0.0;
        } else {
          q_t_i =  q_min[(i - k)*m + (j - k)];
        }
        q_t[i*SIZE + j] = q_t_i;
      }
    }

    if (k == 0) {
      memcpy(Q, q_t, sizeof(float) * SIZE * SIZE);     // Q = q_t
      naive_fixed_matrix_multiply(q_t, A, R); // R = q_t * A
    } else {
      float *res = (float *)calloc(sizeof(float), SIZE * SIZE);
      naive_fixed_matrix_multiply(q_t, Q, res); // R = q_t * A
      memcpy(Q, res, sizeof(float) * SIZE * SIZE);
      naive_fixed_matrix_multiply(q_t, R, res); // R = q_t * A
      memcpy(R, res, sizeof(float) * SIZE * SIZE);
    }
    free(x);
    free(e);
    free(u);
    free(v);
    free(q_min);
    free(q_t);
  }
  naive_fixed_transpose(Q);
}


int main(int argc, char **argv) {

  FILE *file = fopen(OUTFILE, "w");
  if (file == NULL) file = stdout;

  fprintf(file, "kernel,SIZE,cycles,correct\n");

  init_rand(10);

  create_random_mat(a, SIZE, SIZE);
  zero_matrix(q, SIZE, SIZE);
  zero_matrix(r, SIZE, SIZE);
  zero_matrix(q_spec, SIZE, SIZE);
  zero_matrix(r_spec, SIZE, SIZE);

  print_matrix(a, SIZE, SIZE);

  int err;
  int time = 0;

  printf("Starting spec run\n");
  naive_qr_decomp(a, q_spec, r_spec, SIZE);

  // Diospyros
  start_cycle_timing;
  kernel(a, q, r);
  stop_cycle_timing;
  time = get_time();
  print_matrix(q, SIZE, SIZE);
  print_matrix(r, SIZE, SIZE);
  printf("Compgen : %d cycles\n", time);
  bool q_correct = output_check_abs(q, q_spec, SIZE, SIZE);
  bool r_correct = output_check_abs(r, r_spec, SIZE, SIZE);
  fprintf(file, "%s,%d,%d,%d\n","compgen", SIZE, time, q_correct && r_correct);

  return 0;
}
