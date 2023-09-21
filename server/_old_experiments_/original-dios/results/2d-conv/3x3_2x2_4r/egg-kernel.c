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
int __attribute__((section(".dram0.data"))) v_5[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_5_0[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_8[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_8_0[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_10[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_10_0[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_15[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_15_0[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_24[4] = {0, 4, 5, 2};
int __attribute__((section(".dram0.data"))) v_24_0[4] = {0, 4, 5, 2};
int __attribute__((section(".dram0.data"))) v_26[4] = {2, 0, 0, 3};
int __attribute__((section(".dram0.data"))) v_26_0[4] = {2, 0, 0, 3};
int __attribute__((section(".dram0.data"))) v_30[4] = {3, 0, 1, 5};
int __attribute__((section(".dram0.data"))) v_30_0[4] = {3, 0, 1, 5};
int __attribute__((section(".dram0.data"))) v_32[4] = {0, 3, 3, 1};
int __attribute__((section(".dram0.data"))) v_32_0[4] = {0, 3, 3, 1};
int __attribute__((section(".dram0.data"))) v_46[4] = {3, 7, 8, 5};
int __attribute__((section(".dram0.data"))) v_46_0[4] = {0, 1, 4, 3};
int __attribute__((section(".dram0.data"))) v_46_0_0[4] = {3, 7, 0, 5};
int __attribute__((section(".dram0.data"))) v_52[4] = {6, 3, 4, 8};
int __attribute__((section(".dram0.data"))) v_52_0[4] = {0, 1, 2, 4};
int __attribute__((section(".dram0.data"))) v_52_0_0[4] = {2, 7, 0, 0};
int __attribute__((section(".dram0.data"))) v_58[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_58_0[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_61[4] = {6, 7, 8, 8};
int __attribute__((section(".dram0.data"))) v_61_0[4] = {2, 3, 4, 4};
int __attribute__((section(".dram0.data"))) v_63[4] = {2, 2, 2, 3};
int __attribute__((section(".dram0.data"))) v_63_0[4] = {2, 2, 2, 3};
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
  float v_13 = I[2];
  float v_14_tmp[4] = {v_1, v_3, v_13, v_1};
  xb_vecMxf32 v_14 = *((xb_vecMxf32 *) v_14_tmp);
  xb_vecMxf32 v_16;
  v_16 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_15_0)));
  xb_vecMxf32 v_17 = PDX_MUL_MXF32(v_14, v_16);
  float v_18 = I[3];
  float v_19 = I[4];
  float v_20_tmp[4] = {v_1, v_18, v_19, v_1};
  xb_vecMxf32 v_20 = *((xb_vecMxf32 *) v_20_tmp);
  xb_vecMxf32 v_23 = PDX_MUL_MXF32(v_20, v_6);
  xb_vecMxf32 v_25;
  v_25 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_24_0)));
  xb_vecMxf32 v_27;
  v_27 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_26_0)));
  xb_vecMxf32 v_28 = v_23;
  PDX_MULA_MXF32(v_28, v_25, v_27);
  xb_vecMxf32 v_29 = PDX_ADD_MXF32(v_17, v_28);
  xb_vecMxf32 v_31;
  v_31 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_30_0)));
  xb_vecMxf32 v_33;
  v_33 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_32_0)));
  xb_vecMxf32 v_34 = v_29;
  PDX_MULA_MXF32(v_34, v_31, v_33);
  float v_35 = I[5];
  float v_36_tmp[4] = {v_1, v_19, v_35, v_1};
  xb_vecMxf32 v_36 = *((xb_vecMxf32 *) v_36_tmp);
  xb_vecMxf32 v_39 = PDX_MUL_MXF32(v_36, v_16);
  float v_40 = I[6];
  float v_41 = I[7];
  float v_42_tmp[4] = {v_1, v_40, v_41, v_1};
  xb_vecMxf32 v_42 = *((xb_vecMxf32 *) v_42_tmp);
  xb_vecMxf32 v_45 = PDX_MUL_MXF32(v_42, v_6);
  xb_vecMxf32 v_47_tmp_0;
  v_47_tmp_0 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_46_0_0)));
  xb_vecMxf32 v_47;
  v_47 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(v_47_tmp_0), *((xb_vecMx32 *) v_46_0)));
  xb_vecMxf32 v_50 = v_45;
  PDX_MULA_MXF32(v_50, v_47, v_27);
  xb_vecMxf32 v_51 = PDX_ADD_MXF32(v_39, v_50);
  xb_vecMxf32 v_53_tmp_0;
  v_53_tmp_0 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMx32 *) v_52_0_0)));
  xb_vecMxf32 v_53;
  v_53 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(v_53_tmp_0), *((xb_vecMx32 *) v_52_0)));
  xb_vecMxf32 v_56 = v_51;
  PDX_MULA_MXF32(v_56, v_53, v_33);
  xb_vecMxf32 v_59;
  v_59 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_58_0)));
  xb_vecMxf32 v_60 = PDX_MUL_MXF32(v_42, v_59);
  xb_vecMxf32 v_62;
  v_62 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMx32 *) v_61_0)));
  xb_vecMxf32 v_64;
  v_64 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_63_0)));
  xb_vecMxf32 v_65 = v_60;
  PDX_MULA_MXF32(v_65, v_62, v_64);
  PDX_SAV_MXF32_XP(v_12, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_34, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_56, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_65, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAPOS_MXF32_FP(align_O, (xb_vecMxf32 *) O);
}