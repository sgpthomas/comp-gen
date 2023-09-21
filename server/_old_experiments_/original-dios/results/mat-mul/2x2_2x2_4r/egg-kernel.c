#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>
/*
Git revision: 09bd97a

Git status clean
*/
int __attribute__((section(".dram0.data"))) Z[4] = {0, 0, 0, 0};
float __attribute__((section(".dram0.data"))) v_0[4] = {0.0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_1[4] = {0, 0, 2, 2};
int __attribute__((section(".dram0.data"))) v_1_0[4] = {0, 0, 2, 2};
int __attribute__((section(".dram0.data"))) v_3[4] = {0, 1, 0, 1};
int __attribute__((section(".dram0.data"))) v_3_0[4] = {0, 1, 0, 1};
int __attribute__((section(".dram0.data"))) v_6[4] = {1, 1, 3, 3};
int __attribute__((section(".dram0.data"))) v_6_0[4] = {1, 1, 3, 3};
int __attribute__((section(".dram0.data"))) v_8[4] = {2, 3, 2, 3};
int __attribute__((section(".dram0.data"))) v_8_0[4] = {2, 3, 2, 3};
void kernel(float * A, float * B, float * C) {
float * __restrict A_mut = A;
  valign align_A;
  align_A = PDX_LA_MXF32_PP((xb_vecMxf32 *) A);
  float * __restrict B_mut = B;
  valign align_B;
  align_B = PDX_LA_MXF32_PP((xb_vecMxf32 *) B);
  float * __restrict C_mut = C;
  valign align_C = PDX_Z_ALIGN();
  xb_vecMxf32 A_0_4;
  PDX_LAV_MXF32_XP(A_0_4, align_A, (xb_vecMxf32 *) A_mut, 16);
  xb_vecMxf32 B_0_4;
  PDX_LAV_MXF32_XP(B_0_4, align_B, (xb_vecMxf32 *) B_mut, 16);
  xb_vecMxf32 v_2;
  v_2 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_1_0)));
  xb_vecMxf32 v_4;
  v_4 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(B_0_4), *((xb_vecMx32 *) v_3_0)));
  xb_vecMxf32 v_5 = PDX_MUL_MXF32(v_2, v_4);
  xb_vecMxf32 v_7;
  v_7 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_6_0)));
  xb_vecMxf32 v_9;
  v_9 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(B_0_4), *((xb_vecMx32 *) v_8_0)));
  xb_vecMxf32 v_10 = v_5;
  PDX_MULA_MXF32(v_10, v_7, v_9);
  PDX_SAV_MXF32_XP(v_10, align_C, (xb_vecMxf32 *) C, 16);
  PDX_SAPOS_MXF32_FP(align_C, (xb_vecMxf32 *) C);
}