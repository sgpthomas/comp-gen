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
int __attribute__((section(".dram0.data"))) v_1[4] = {1, 1, 1, 1};
int __attribute__((section(".dram0.data"))) v_1_0[4] = {1, 1, 1, 1};
int __attribute__((section(".dram0.data"))) v_3[4] = {4, 5, 6, 7};
int __attribute__((section(".dram0.data"))) v_6[4] = {2, 2, 2, 2};
int __attribute__((section(".dram0.data"))) v_6_0[4] = {2, 2, 2, 2};
int __attribute__((section(".dram0.data"))) v_8[4] = {8, 9, 10, 11};
int __attribute__((section(".dram0.data"))) v_11[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_11_0[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_13[4] = {12, 13, 14, 15};
int __attribute__((section(".dram0.data"))) v_17[4] = {0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_17_0[4] = {0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_19[4] = {0, 1, 2, 3};
int __attribute__((section(".dram0.data"))) v_22[4] = {5, 5, 5, 5};
int __attribute__((section(".dram0.data"))) v_22_0[4] = {1, 1, 1, 1};
int __attribute__((section(".dram0.data"))) v_27[4] = {6, 6, 6, 6};
int __attribute__((section(".dram0.data"))) v_27_0[4] = {2, 2, 2, 2};
int __attribute__((section(".dram0.data"))) v_32[4] = {7, 7, 7, 7};
int __attribute__((section(".dram0.data"))) v_32_0[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_38[4] = {4, 4, 4, 4};
int __attribute__((section(".dram0.data"))) v_38_0[4] = {0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_43[4] = {9, 9, 9, 9};
int __attribute__((section(".dram0.data"))) v_43_0[4] = {1, 1, 1, 1};
int __attribute__((section(".dram0.data"))) v_48[4] = {10, 10, 10, 10};
int __attribute__((section(".dram0.data"))) v_48_0[4] = {2, 2, 2, 2};
int __attribute__((section(".dram0.data"))) v_53[4] = {11, 11, 11, 11};
int __attribute__((section(".dram0.data"))) v_53_0[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_59[4] = {8, 8, 8, 8};
int __attribute__((section(".dram0.data"))) v_59_0[4] = {0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_64[4] = {13, 13, 13, 13};
int __attribute__((section(".dram0.data"))) v_64_0[4] = {1, 1, 1, 1};
int __attribute__((section(".dram0.data"))) v_69[4] = {14, 14, 14, 14};
int __attribute__((section(".dram0.data"))) v_69_0[4] = {2, 2, 2, 2};
int __attribute__((section(".dram0.data"))) v_74[4] = {15, 15, 15, 15};
int __attribute__((section(".dram0.data"))) v_74_0[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_80[4] = {12, 12, 12, 12};
int __attribute__((section(".dram0.data"))) v_80_0[4] = {0, 0, 0, 0};
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
  xb_vecMxf32 A_4_8;
  PDX_LAV_MXF32_XP(A_4_8, align_A, (xb_vecMxf32 *) A_mut, 16);
  xb_vecMxf32 A_8_12;
  PDX_LAV_MXF32_XP(A_8_12, align_A, (xb_vecMxf32 *) A_mut, 16);
  xb_vecMxf32 A_12_16;
  PDX_LAV_MXF32_XP(A_12_16, align_A, (xb_vecMxf32 *) A_mut, 16);
  xb_vecMxf32 B_0_4;
  PDX_LAV_MXF32_XP(B_0_4, align_B, (xb_vecMxf32 *) B_mut, 16);
  xb_vecMxf32 B_4_8;
  PDX_LAV_MXF32_XP(B_4_8, align_B, (xb_vecMxf32 *) B_mut, 16);
  xb_vecMxf32 B_8_12;
  PDX_LAV_MXF32_XP(B_8_12, align_B, (xb_vecMxf32 *) B_mut, 16);
  xb_vecMxf32 B_12_16;
  PDX_LAV_MXF32_XP(B_12_16, align_B, (xb_vecMxf32 *) B_mut, 16);
  xb_vecMxf32 v_2;
  v_2 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_1_0)));
  xb_vecMxf32 v_4;
  v_4 = B_4_8;
  xb_vecMxf32 v_5 = PDX_MUL_MXF32(v_2, v_4);
  xb_vecMxf32 v_7;
  v_7 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_6_0)));
  xb_vecMxf32 v_9;
  v_9 = B_8_12;
  xb_vecMxf32 v_10 = PDX_MUL_MXF32(v_7, v_9);
  xb_vecMxf32 v_12;
  v_12 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_11_0)));
  xb_vecMxf32 v_14;
  v_14 = B_12_16;
  xb_vecMxf32 v_15 = v_10;
  PDX_MULA_MXF32(v_15, v_12, v_14);
  xb_vecMxf32 v_16 = PDX_ADD_MXF32(v_5, v_15);
  xb_vecMxf32 v_18;
  v_18 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_0_4), *((xb_vecMx32 *) v_17_0)));
  xb_vecMxf32 v_20;
  v_20 = B_0_4;
  xb_vecMxf32 v_21 = v_16;
  PDX_MULA_MXF32(v_21, v_18, v_20);
  xb_vecMxf32 v_23;
  v_23 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_4_8), *((xb_vecMx32 *) v_22_0)));
  xb_vecMxf32 v_26 = PDX_MUL_MXF32(v_23, v_4);
  xb_vecMxf32 v_28;
  v_28 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_4_8), *((xb_vecMx32 *) v_27_0)));
  xb_vecMxf32 v_31 = PDX_MUL_MXF32(v_28, v_9);
  xb_vecMxf32 v_33;
  v_33 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_4_8), *((xb_vecMx32 *) v_32_0)));
  xb_vecMxf32 v_36 = v_31;
  PDX_MULA_MXF32(v_36, v_33, v_14);
  xb_vecMxf32 v_37 = PDX_ADD_MXF32(v_26, v_36);
  xb_vecMxf32 v_39;
  v_39 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_4_8), *((xb_vecMx32 *) v_38_0)));
  xb_vecMxf32 v_42 = v_37;
  PDX_MULA_MXF32(v_42, v_39, v_20);
  xb_vecMxf32 v_44;
  v_44 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_8_12), *((xb_vecMx32 *) v_43_0)));
  xb_vecMxf32 v_47 = PDX_MUL_MXF32(v_44, v_4);
  xb_vecMxf32 v_49;
  v_49 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_8_12), *((xb_vecMx32 *) v_48_0)));
  xb_vecMxf32 v_52 = PDX_MUL_MXF32(v_49, v_9);
  xb_vecMxf32 v_54;
  v_54 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_8_12), *((xb_vecMx32 *) v_53_0)));
  xb_vecMxf32 v_57 = v_52;
  PDX_MULA_MXF32(v_57, v_54, v_14);
  xb_vecMxf32 v_58 = PDX_ADD_MXF32(v_47, v_57);
  xb_vecMxf32 v_60;
  v_60 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_8_12), *((xb_vecMx32 *) v_59_0)));
  xb_vecMxf32 v_63 = v_58;
  PDX_MULA_MXF32(v_63, v_60, v_20);
  xb_vecMxf32 v_65;
  v_65 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_12_16), *((xb_vecMx32 *) v_64_0)));
  xb_vecMxf32 v_68 = PDX_MUL_MXF32(v_65, v_4);
  xb_vecMxf32 v_70;
  v_70 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_12_16), *((xb_vecMx32 *) v_69_0)));
  xb_vecMxf32 v_73 = PDX_MUL_MXF32(v_70, v_9);
  xb_vecMxf32 v_75;
  v_75 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_12_16), *((xb_vecMx32 *) v_74_0)));
  xb_vecMxf32 v_78 = v_73;
  PDX_MULA_MXF32(v_78, v_75, v_14);
  xb_vecMxf32 v_79 = PDX_ADD_MXF32(v_68, v_78);
  xb_vecMxf32 v_81;
  v_81 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(A_12_16), *((xb_vecMx32 *) v_80_0)));
  xb_vecMxf32 v_84 = v_79;
  PDX_MULA_MXF32(v_84, v_81, v_20);
  PDX_SAV_MXF32_XP(v_21, align_C, (xb_vecMxf32 *) C, 16);
  PDX_SAV_MXF32_XP(v_42, align_C, (xb_vecMxf32 *) C, 16);
  PDX_SAV_MXF32_XP(v_63, align_C, (xb_vecMxf32 *) C, 16);
  PDX_SAV_MXF32_XP(v_84, align_C, (xb_vecMxf32 *) C, 16);
  PDX_SAPOS_MXF32_FP(align_C, (xb_vecMxf32 *) C);
}