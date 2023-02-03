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
int __attribute__((section(".dram0.data"))) v_1[4] = {0, 4, 5, 0};
int __attribute__((section(".dram0.data"))) v_1_0[4] = {0, 4, 5, 0};
int __attribute__((section(".dram0.data"))) v_3[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_3_0[4] = {0, 5, 5, 0};
int __attribute__((section(".dram0.data"))) v_6[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_6_0[4] = {0, 1, 2, 2};
int __attribute__((section(".dram0.data"))) v_8[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_8_0[4] = {0, 0, 0, 1};
int __attribute__((section(".dram0.data"))) v_11[4] = {0, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_11_0[4] = {0, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_13[4] = {2, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_13_0[4] = {2, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_16[4] = {0, 5, 6, 0};
int __attribute__((section(".dram0.data"))) v_16_0[4] = {0, 5, 6, 0};
int __attribute__((section(".dram0.data"))) v_18[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_18_0[4] = {0, 6, 6, 0};
int __attribute__((section(".dram0.data"))) v_21[4] = {0, 7, 8, 0};
int __attribute__((section(".dram0.data"))) v_21_0[4] = {0, 1, 4, 3};
int __attribute__((section(".dram0.data"))) v_21_0_0[4] = {0, 7, 0, 0};
int __attribute__((section(".dram0.data"))) v_26[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_26_0[4] = {3, 4, 5, 5};
int __attribute__((section(".dram0.data"))) v_33[4] = {3, 3, 4, 5};
int __attribute__((section(".dram0.data"))) v_33_0[4] = {3, 3, 4, 5};
int __attribute__((section(".dram0.data"))) v_38[4] = {0, 8, 9, 0};
int __attribute__((section(".dram0.data"))) v_38_0[4] = {0, 4, 5, 0};
int __attribute__((section(".dram0.data"))) v_43[4] = {0, 10, 11, 0};
int __attribute__((section(".dram0.data"))) v_43_0[4] = {0, 6, 7, 0};
int __attribute__((section(".dram0.data"))) v_48[4] = {6, 7, 8, 8};
int __attribute__((section(".dram0.data"))) v_48_0[4] = {2, 3, 4, 4};
int __attribute__((section(".dram0.data"))) v_57[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_57_0[4] = {0, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_62[4] = {2, 2, 2, 3};
int __attribute__((section(".dram0.data"))) v_62_0[4] = {2, 2, 2, 3};
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
  xb_vecMxf32 v_2;
  v_2 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_1_0)));
  xb_vecMxf32 v_4;
  v_4 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_3_0)));
  xb_vecMxf32 v_5 = PDX_MUL_MXF32(v_2, v_4);
  xb_vecMxf32 v_7;
  v_7 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_6_0)));
  xb_vecMxf32 v_9;
  v_9 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_8_0)));
  xb_vecMxf32 v_10 = v_5;
  PDX_MULA_MXF32(v_10, v_7, v_9);
  xb_vecMxf32 v_12;
  v_12 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_11_0)));
  xb_vecMxf32 v_14;
  v_14 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_13_0)));
  xb_vecMxf32 v_15 = PDX_MUL_MXF32(v_12, v_14);
  xb_vecMxf32 v_17;
  v_17 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_16_0)));
  xb_vecMxf32 v_19;
  v_19 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_18_0)));
  xb_vecMxf32 v_20 = PDX_MUL_MXF32(v_17, v_19);
  xb_vecMxf32 v_22_tmp_0;
  v_22_tmp_0 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_21_0_0)));
  xb_vecMxf32 v_22;
  v_22 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(v_22_tmp_0), *((xb_vecMx32 *) v_21_0)));
  xb_vecMxf32 v_25 = PDX_MUL_MXF32(v_22, v_4);
  xb_vecMxf32 v_27;
  v_27 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_26_0)));
  xb_vecMxf32 v_30 = v_25;
  PDX_MULA_MXF32(v_30, v_27, v_9);
  xb_vecMxf32 v_31 = PDX_ADD_MXF32(v_20, v_30);
  xb_vecMxf32 v_32 = PDX_ADD_MXF32(v_15, v_31);
  xb_vecMxf32 v_34;
  v_34 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), PDX_MOV_MX32_FROM_MXF32(I_0_4), *((xb_vecMx32 *) v_33_0)));
  xb_vecMxf32 v_37 = PDX_MUL_MXF32(v_34, v_14);
  xb_vecMxf32 v_39;
  v_39 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_38_0)));
  xb_vecMxf32 v_42 = PDX_MUL_MXF32(v_39, v_19);
  xb_vecMxf32 v_44;
  v_44 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_43_0)));
  xb_vecMxf32 v_47 = PDX_MUL_MXF32(v_44, v_4);
  xb_vecMxf32 v_49;
  v_49 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(I_8_12), PDX_MOV_MX32_FROM_MXF32(I_4_8), *((xb_vecMx32 *) v_48_0)));
  xb_vecMxf32 v_52 = v_47;
  PDX_MULA_MXF32(v_52, v_49, v_9);
  xb_vecMxf32 v_53 = PDX_ADD_MXF32(v_42, v_52);
  xb_vecMxf32 v_54 = PDX_ADD_MXF32(v_37, v_53);
  xb_vecMxf32 v_58;
  v_58 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_57_0)));
  xb_vecMxf32 v_59 = PDX_MUL_MXF32(v_44, v_58);
  xb_vecMxf32 v_63;
  v_63 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(F_0_4), *((xb_vecMx32 *) v_62_0)));
  xb_vecMxf32 v_64 = v_59;
  PDX_MULA_MXF32(v_64, v_49, v_63);
  PDX_SAV_MXF32_XP(v_10, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_32, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_54, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAV_MXF32_XP(v_64, align_O, (xb_vecMxf32 *) O, 16);
  PDX_SAPOS_MXF32_FP(align_O, (xb_vecMxf32 *) O);
}