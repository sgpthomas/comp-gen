#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <xtensa/sim.h>
#include <xtensa/tie/xt_pdxn.h>
#include <xtensa/tie/xt_timer.h>
#include <xtensa/xt_profiling.h>

int __attribute__((section(".dram0.data"))) Z[4] = {0, 0, 0, 0};
float __attribute__((section(".dram0.data"))) v_0[4] = {0.0, 0, 0, 0};
int __attribute__((section(".dram0.data"))) v_5[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_5_0[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_8[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_8_0[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_10[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_10_0[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_13[4] = {0, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_13_0[4] = {0, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_15[4] = {2, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_15_0[4] = {2, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_20[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_20_0[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_29[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_29_0[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_36[4] = {3, 3, 4, 5};
int __attribute__((section(".dram0.data"))) v_36_0[4] = {3, 3, 4, 5};
int __attribute__((section(".dram0.data"))) v_41[4] = {0, 8, 9, 0};
int __attribute__((section(".dram0.data"))) v_41_0[4] = {0, 4, 5, 0};
int __attribute__((section(".dram0.data"))) v_46[4] = {0, 10, 11, 0};
int __attribute__((section(".dram0.data"))) v_46_0[4] = {0, 6, 7, 0};
int __attribute__((section(".dram0.data"))) v_51[4] = {6, 7, 8, 8};
int __attribute__((section(".dram0.data"))) v_51_0[4] = {2, 3, 4, 4};
int __attribute__((section(".dram0.data"))) v_60[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_60_0[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_65[4] = {2, 2, 2, 3};
int __attribute__((section(".dram0.data"))) v_65_0[4] = {2, 2, 2, 3};
void kernel(float * I, float * F, float * O) {
float * __restrict I_mut = I;
  valign align_I;
  align_I = PDX_LA_MXF32_PP((xb_vecMxf32 *) I);
  float * __restrict F_mut = F;
  valign align_F;
  align_F = PDX_LA_MXF32_PP((xb_vecMxf32 *) F);
  float * __restrict O_mut = O;
  valign align_O = PDX_Z_ALIGN();
  xb_vecMxf32 I_0_4;
  PDX_LAV_MXF32_XP(I_0_4, align_I, (xb_vecMxf32 *) I_mut, 16);
  xb_vecMxf32 I_4_8;
  PDX_LAV_MXF32_XP(I_4_8, align_I, (xb_vecMxf32 *) I_mut, 16);
  xb_vecMxf32 I_8_12;
  PDX_LAV_MXF32_XP(I_8_12, align_I, (xb_vecMxf32 *) I_mut, 16);
  xb_vecMxf32 F_0_4;
  PDX_LAV_MXF32_XP(F_0_4, align_F, (xb_vecMxf32 *) F_mut, 16);
  float v_1 = 1;
  float v_2 = I[0];
  float v_3 = I[1];
  float v_4_tmp[4] = {v_1, v_2, v_3, v_1};
  xb_vecMxf32 v_4 = *((xb_vecMxf32 *) v_4_tmp);
  xb_vecMxf32 v_6;
  v_6 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_5_0)));
  xb_vecMxf32 v_7 = PDX_MUL_MXF32(v_4, v_6);
  xb_vecMxf32 v_9;
  v_9 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_8_0)));
  xb_vecMxf32 v_11;
  v_11 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_10_0)));
  xb_vecMxf32 v_12 = v_7;
  PDX_MULA_MXF32(v_12, v_9, v_11);
  xb_vecMxf32 v_14;
  v_14 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_13_0)));
  xb_vecMxf32 v_16;
  v_16 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_15_0)));
  xb_vecMxf32 v_17 = PDX_MUL_MXF32(v_14, v_16);
  float v_18 = I[2];
  float v_19_tmp[4] = {v_1, v_3, v_18, v_1};
  xb_vecMxf32 v_19 = *((xb_vecMxf32 *) v_19_tmp);
  xb_vecMxf32 v_21;
  v_21 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_20_0)));
  xb_vecMxf32 v_22 = PDX_MUL_MXF32(v_19, v_21);
  float v_23 = I[3];
  float v_24 = I[4];
  float v_25_tmp[4] = {v_1, v_23, v_24, v_1};
  xb_vecMxf32 v_25 = *((xb_vecMxf32 *) v_25_tmp);
  xb_vecMxf32 v_28 = PDX_MUL_MXF32(v_25, v_6);
  xb_vecMxf32 v_30;
  v_30 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_29_0)));
  xb_vecMxf32 v_33 = v_28;
  PDX_MULA_MXF32(v_33, v_30, v_11);
  xb_vecMxf32 v_34 = PDX_ADD_MXF32(v_22, v_33);
  xb_vecMxf32 v_35 = PDX_ADD_MXF32(v_17, v_34);
  xb_vecMxf32 v_37;
  v_37 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_36_0)));
  xb_vecMxf32 v_40 = PDX_MUL_MXF32(v_37, v_16);
  xb_vecMxf32 v_42;
  v_42 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_41_0)));
  xb_vecMxf32 v_45 = PDX_MUL_MXF32(v_42, v_21);
  xb_vecMxf32 v_47;
  v_47 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_46_0)));
  xb_vecMxf32 v_50 = PDX_MUL_MXF32(v_47, v_6);
  xb_vecMxf32 v_52;
  v_52 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMx32 *) v_51_0)));
  xb_vecMxf32 v_55 = v_50;
  PDX_MULA_MXF32(v_55, v_52, v_11);
  xb_vecMxf32 v_56 = PDX_ADD_MXF32(v_45, v_55);
  xb_vecMxf32 v_57 = PDX_ADD_MXF32(v_40, v_56);
  xb_vecMxf32 v_61;
  v_61 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_60_0)));
  xb_vecMxf32 v_62 = PDX_MUL_MXF32(v_47, v_61);
  xb_vecMxf32 v_66;
  v_66 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_65_0)));
  xb_vecMxf32 v_67 = v_62;
  PDX_MULA_MXF32(v_67, v_52, v_66);
  PDX_SAV_MXF32_XP(v_12, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_35, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_57, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_67, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAPOS_MXF32_FP(align_O, (xb_vecMxf32 *) O);
}