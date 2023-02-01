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
int __attribute__((section(".dram0.data"))) v_8[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_8_0[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_10[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_10_0[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_25[4] = {0, 5, 6, 0};
int __attribute__((section(".dram0.data"))) v_25_0[4] = {0, 5, 6, 0};
int __attribute__((section(".dram0.data"))) v_29[4] = {0, 3, 1, 1};
int __attribute__((section(".dram0.data"))) v_29_0[4] = {0, 3, 1, 1};
int __attribute__((section(".dram0.data"))) v_31[4] = {3, 0, 4, 5};
int __attribute__((section(".dram0.data"))) v_31_0[4] = {3, 0, 4, 5};
int __attribute__((section(".dram0.data"))) v_34[4] = {0, 1, 5, 2};
int __attribute__((section(".dram0.data"))) v_34_0[4] = {0, 1, 5, 2};
int __attribute__((section(".dram0.data"))) v_36[4] = {2, 2, 0, 3};
int __attribute__((section(".dram0.data"))) v_36_0[4] = {2, 2, 0, 3};
int __attribute__((section(".dram0.data"))) v_44[4] = {2, 2, 2, 3};
int __attribute__((section(".dram0.data"))) v_44_0[4] = {2, 2, 2, 3};
int __attribute__((section(".dram0.data"))) v_46[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_46_0[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_50[4] = {0, 11, 11, 0};
int __attribute__((section(".dram0.data"))) v_50_0[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_53[4] = {6, 6, 8, 8};
int __attribute__((section(".dram0.data"))) v_53_0[4] = {2, 2, 4, 4};
int __attribute__((section(".dram0.data"))) v_55[4] = {0, 1, 0, 1};
int __attribute__((section(".dram0.data"))) v_55_0[4] = {0, 1, 0, 1};
int __attribute__((section(".dram0.data"))) v_65[4] = {2, 3, 2, 3};
int __attribute__((section(".dram0.data"))) v_65_0[4] = {2, 3, 2, 3};
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
  float v_3 = F[1];
  float v_4_tmp[4] = {v_1, v_2, v_3, v_1};
  xb_vecMxf32 v_4 = *((xb_vecMxf32 *) v_4_tmp);
  xb_vecMxf32 v_6;
  v_6 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_5_0)));
  xb_vecMxf32 v_7 = PDX_MUL_MXF32(v_4, v_6);
  xb_vecMxf32 v_9;
  v_9 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_8_0)));
  xb_vecMxf32 v_11;
  v_11 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_10_0)));
  xb_vecMxf32 v_12 = PDX_MUL_MXF32(v_9, v_11);
  xb_vecMxf32 v_13 = PDX_ADD_MXF32(v_7, v_12);
  float v_14 = 0;
  float v_15 = F[0];
  float v_16 = I[1];
  float v_17_tmp[4] = {v_14, v_15, v_16, v_1};
  xb_vecMxf32 v_17 = *((xb_vecMxf32 *) v_17_tmp);
  float v_18 = I[4];
  float v_19 = F[3];
  float v_20_tmp[4] = {v_1, v_18, v_19, v_14};
  xb_vecMxf32 v_20 = *((xb_vecMxf32 *) v_20_tmp);
  xb_vecMxf32 v_21 = PDX_MUL_MXF32(v_17, v_20);
  float v_22 = I[3];
  float v_23 = F[2];
  float v_24_tmp[4] = {v_1, v_22, v_23, v_1};
  xb_vecMxf32 v_24 = *((xb_vecMxf32 *) v_24_tmp);
  xb_vecMxf32 v_26;
  v_26 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_25_0)));
  xb_vecMxf32 v_27 = PDX_MUL_MXF32(v_24, v_26);
  xb_vecMxf32 v_28 = PDX_ADD_MXF32(v_21, v_27);
  xb_vecMxf32 v_30;
  v_30 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_29_0)));
  xb_vecMxf32 v_32;
  v_32 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_31_0)));
  xb_vecMxf32 v_33 = PDX_MUL_MXF32(v_30, v_32);
  xb_vecMxf32 v_35;
  v_35 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_34_0)));
  xb_vecMxf32 v_37;
  v_37 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_36_0)));
  xb_vecMxf32 v_38 = PDX_MUL_MXF32(v_35, v_37);
  xb_vecMxf32 v_39 = PDX_ADD_MXF32(v_33, v_38);
  xb_vecMxf32 v_40 = PDX_ADD_MXF32(v_28, v_39);
  float v_41_tmp[4] = {v_14, v_19, v_19, v_1};
  xb_vecMxf32 v_41 = *((xb_vecMxf32 *) v_41_tmp);
  float v_42_tmp[4] = {v_1, v_22, v_18, v_14};
  xb_vecMxf32 v_42 = *((xb_vecMxf32 *) v_42_tmp);
  xb_vecMxf32 v_43 = PDX_MUL_MXF32(v_41, v_42);
  xb_vecMxf32 v_45;
  v_45 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_44_0)));
  xb_vecMxf32 v_47;
  v_47 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_46_0)));
  xb_vecMxf32 v_48 = PDX_MUL_MXF32(v_45, v_47);
  float v_49_tmp[4] = {v_1, v_15, v_3, v_1};
  xb_vecMxf32 v_49 = *((xb_vecMxf32 *) v_49_tmp);
  xb_vecMxf32 v_51;
  v_51 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_50_0)));
  xb_vecMxf32 v_52 = PDX_MUL_MXF32(v_49, v_51);
  xb_vecMxf32 v_54;
  v_54 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMx32 *) v_53_0)));
  xb_vecMxf32 v_56;
  v_56 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_55_0)));
  xb_vecMxf32 v_57 = PDX_MUL_MXF32(v_54, v_56);
  xb_vecMxf32 v_58 = PDX_ADD_MXF32(v_52, v_57);
  xb_vecMxf32 v_59 = PDX_ADD_MXF32(v_48, v_58);
  xb_vecMxf32 v_60 = PDX_ADD_MXF32(v_43, v_59);
  float v_61_tmp[4] = {v_1, v_23, v_19, v_1};
  xb_vecMxf32 v_61 = *((xb_vecMxf32 *) v_61_tmp);
  xb_vecMxf32 v_64 = PDX_MUL_MXF32(v_61, v_51);
  xb_vecMxf32 v_66;
  v_66 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_65_0)));
  xb_vecMxf32 v_69 = PDX_MUL_MXF32(v_66, v_54);
  xb_vecMxf32 v_70 = PDX_ADD_MXF32(v_64, v_69);
  PDX_SAV_MXF32_XP(v_13, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_40, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_60, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_70, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAPOS_MXF32_FP(align_O, (xb_vecMxf32 *) O);
}