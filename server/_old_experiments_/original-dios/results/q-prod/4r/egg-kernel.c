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
int __attribute__((section(".dram0.data"))) v_4[4] = {4, 5, 6, 0};
int __attribute__((section(".dram0.data"))) v_4_0[4] = {0, 1, 2, 4};
int __attribute__((section(".dram0.data"))) v_11[4] = {7, 7, 7, 0};
int __attribute__((section(".dram0.data"))) v_11_0[4] = {3, 3, 3, 4};
int __attribute__((section(".dram0.data"))) v_14[4] = {1, 2, 0, 3};
int __attribute__((section(".dram0.data"))) v_14_0[4] = {1, 2, 0, 3};
int __attribute__((section(".dram0.data"))) v_16[4] = {2, 0, 1, 3};
int __attribute__((section(".dram0.data"))) v_16_0[4] = {2, 0, 1, 3};
int __attribute__((section(".dram0.data"))) v_21[4] = {0, 0, 0, 4};
int __attribute__((section(".dram0.data"))) v_21_0[4] = {0, 0, 0, 4};
int __attribute__((section(".dram0.data"))) v_26[4] = {0, 0, 0, 5};
int __attribute__((section(".dram0.data"))) v_26_0[4] = {0, 0, 0, 5};
int __attribute__((section(".dram0.data"))) v_30[4] = {2, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_30_0[4] = {2, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_32[4] = {1, 2, 0, 2};
int __attribute__((section(".dram0.data"))) v_32_0[4] = {1, 2, 0, 2};
int __attribute__((section(".dram0.data"))) v_39[4] = {0, 1, 2, 0};
int __attribute__((section(".dram0.data"))) v_39_0[4] = {0, 1, 2, 0};
int __attribute__((section(".dram0.data"))) v_39_1[4] = {0, 1, 2, 0};
int __attribute__((section(".dram0.data"))) v_43[4] = {1, 2, 0, 1};
int __attribute__((section(".dram0.data"))) v_43_0[4] = {1, 2, 0, 1};
int __attribute__((section(".dram0.data"))) v_39_2[4] = {0, 1, 2, 0};
int __attribute__((section(".dram0.data"))) v_43_1[4] = {1, 2, 0, 1};
int __attribute__((section(".dram0.data"))) v_30_1[4] = {2, 0, 1, 2};
int __attribute__((section(".dram0.data"))) v_80[4] = {3, 3, 3, 3};
int __attribute__((section(".dram0.data"))) v_80_0[4] = {3, 3, 3, 3};
void kernel(float * aq, float * at, float * bq, float * bt, float * rq, float * rt) {
float * __restrict aq_mut = aq;
  valign align_aq;
  align_aq = PDX_LA_MXF32_PP((xb_vecMxf32 *) aq);
  float * __restrict at_mut = at;
  valign align_at;
  align_at = PDX_LA_MXF32_PP((xb_vecMxf32 *) at);
  float * __restrict bq_mut = bq;
  valign align_bq;
  align_bq = PDX_LA_MXF32_PP((xb_vecMxf32 *) bq);
  float * __restrict bt_mut = bt;
  valign align_bt;
  align_bt = PDX_LA_MXF32_PP((xb_vecMxf32 *) bt);
  float * __restrict rq_mut = rq;
  valign align_rq = PDX_Z_ALIGN();
  float * __restrict rt_mut = rt;
  valign align_rt = PDX_Z_ALIGN();
  xb_vecMxf32 aq_0_4;
  PDX_LAV_MXF32_XP(aq_0_4, align_aq, (xb_vecMxf32 *) aq_mut, 16);
  xb_vecMxf32 at_0_4;
  PDX_LAV_MXF32_XP(at_0_4, align_at, (xb_vecMxf32 *) at_mut, 16);
  xb_vecMxf32 bq_0_4;
  PDX_LAV_MXF32_XP(bq_0_4, align_bq, (xb_vecMxf32 *) bq_mut, 16);
  xb_vecMxf32 bt_0_4;
  PDX_LAV_MXF32_XP(bt_0_4, align_bt, (xb_vecMxf32 *) bt_mut, 16);
  float v_1 = aq[3];
  float v_2 = 1;
  float v_3_tmp[4] = {v_1, v_1, v_1, v_2};
  xb_vecMxf32 v_3 = *((xb_vecMxf32 *) v_3_tmp);
  xb_vecMxf32 v_5;
  v_5 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(*((xb_vecMxf32 *) v_0), PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMx32 *) v_4_0)));
  xb_vecMxf32 v_6 = PDX_MUL_MXF32(v_3, v_5);
  float v_7 = aq[0];
  float v_8 = aq[1];
  float v_9 = aq[2];
  float v_10_tmp[4] = {v_7, v_8, v_9, v_2};
  xb_vecMxf32 v_10 = *((xb_vecMxf32 *) v_10_tmp);
  xb_vecMxf32 v_12;
  v_12 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(*((xb_vecMxf32 *) v_0), PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMx32 *) v_11_0)));
  xb_vecMxf32 v_13 = PDX_MUL_MXF32(v_10, v_12);
  xb_vecMxf32 v_15;
  v_15 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(aq_0_4), *((xb_vecMx32 *) v_14_0)));
  xb_vecMxf32 v_17;
  v_17 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMx32 *) v_16_0)));
  xb_vecMxf32 v_18 = v_13;
  PDX_MULA_MXF32(v_18, v_15, v_17);
  xb_vecMxf32 v_19 = PDX_ADD_MXF32(v_6, v_18);
  float v_20_tmp[4] = {v_2, v_2, v_2, v_7};
  xb_vecMxf32 v_20 = *((xb_vecMxf32 *) v_20_tmp);
  xb_vecMxf32 v_22;
  v_22 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_21_0)));
  xb_vecMxf32 v_23 = PDX_MUL_MXF32(v_20, v_22);
  xb_vecMxf32 v_24 = PDX_NEG_MXF32(v_23);
  float v_25_tmp[4] = {v_2, v_2, v_2, v_8};
  xb_vecMxf32 v_25 = *((xb_vecMxf32 *) v_25_tmp);
  xb_vecMxf32 v_27;
  v_27 = PDX_MOV_MXF32_FROM_MX32(PDX_SEL_MX32(PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMxf32 *) v_0), *((xb_vecMx32 *) v_26_0)));
  xb_vecMxf32 v_28 = PDX_MUL_MXF32(v_25, v_27);
  xb_vecMxf32 v_29 = PDX_NEG_MXF32(v_28);
  xb_vecMxf32 v_31;
  v_31 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(aq_0_4), *((xb_vecMx32 *) v_30_0)));
  xb_vecMxf32 v_33;
  v_33 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(bq_0_4), *((xb_vecMx32 *) v_32_0)));
  xb_vecMxf32 v_34 = PDX_MUL_MXF32(v_31, v_33);
  xb_vecMxf32 v_35 = PDX_NEG_MXF32(v_34);
  xb_vecMxf32 v_36 = PDX_ADD_MXF32(v_29, v_35);
  xb_vecMxf32 v_37 = PDX_ADD_MXF32(v_24, v_36);
  xb_vecMxf32 v_38 = PDX_ADD_MXF32(v_19, v_37);
  xb_vecMxf32 v_40;
  v_40 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(at_0_4), *((xb_vecMx32 *) v_39_0)));
  xb_vecMxf32 v_42;
  v_42 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(bt_0_4), *((xb_vecMx32 *) v_39_1)));
  xb_vecMxf32 v_44;
  v_44 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(aq_0_4), *((xb_vecMx32 *) v_43_0)));
  float v_45 = 2;
  float v_46_tmp[4] = {v_45, v_45, v_45, v_45};
  xb_vecMxf32 v_46 = *((xb_vecMxf32 *) v_46_tmp);
  xb_vecMxf32 v_48;
  v_48 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(aq_0_4), *((xb_vecMx32 *) v_39_2)));
  xb_vecMxf32 v_50;
  v_50 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(bt_0_4), *((xb_vecMx32 *) v_43_1)));
  xb_vecMxf32 v_51 = PDX_MUL_MXF32(v_48, v_50);
  xb_vecMxf32 v_56 = PDX_MUL_MXF32(v_44, v_42);
  xb_vecMxf32 v_57 = PDX_NEG_MXF32(v_56);
  xb_vecMxf32 v_58 = PDX_ADD_MXF32(v_51, v_57);
  xb_vecMxf32 v_59 = PDX_MUL_MXF32(v_46, v_58);
  xb_vecMxf32 v_60 = PDX_MUL_MXF32(v_44, v_59);
  xb_vecMxf32 v_68 = PDX_MUL_MXF32(v_31, v_42);
  xb_vecMxf32 v_72;
  v_72 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(bt_0_4), *((xb_vecMx32 *) v_30_1)));
  xb_vecMxf32 v_73 = PDX_MUL_MXF32(v_48, v_72);
  xb_vecMxf32 v_74 = PDX_NEG_MXF32(v_73);
  xb_vecMxf32 v_75 = PDX_ADD_MXF32(v_68, v_74);
  xb_vecMxf32 v_76 = PDX_MUL_MXF32(v_46, v_75);
  xb_vecMxf32 v_77 = PDX_MUL_MXF32(v_31, v_76);
  xb_vecMxf32 v_78 = PDX_NEG_MXF32(v_77);
  xb_vecMxf32 v_79 = PDX_ADD_MXF32(v_60, v_78);
  xb_vecMxf32 v_81;
  v_81 = PDX_MOV_MXF32_FROM_MX32(PDX_SHFL_MX32(PDX_MOV_MX32_FROM_MXF32(aq_0_4), *((xb_vecMx32 *) v_80_0)));
  xb_vecMxf32 v_87 = PDX_MUL_MXF32(v_44, v_72);
  xb_vecMxf32 v_92 = PDX_MUL_MXF32(v_31, v_50);
  xb_vecMxf32 v_93 = PDX_NEG_MXF32(v_92);
  xb_vecMxf32 v_94 = PDX_ADD_MXF32(v_87, v_93);
  xb_vecMxf32 v_95 = PDX_MUL_MXF32(v_46, v_94);
  xb_vecMxf32 v_96 = v_79;
  PDX_MULA_MXF32(v_96, v_81, v_95);
  xb_vecMxf32 v_97 = PDX_ADD_MXF32(v_42, v_96);
  xb_vecMxf32 v_98 = PDX_ADD_MXF32(v_40, v_97);
  PDX_SAV_MXF32_XP(v_38, align_rq, (xb_vecMxf32 *) rq, 16);
  PDX_SAV_MXF32_XP(v_98, align_rt, (xb_vecMxf32 *) rt, 12);
  PDX_SAPOS_MXF32_FP(align_rq, (xb_vecMxf32 *) rq);
  PDX_SAPOS_MXF32_FP(align_rt, (xb_vecMxf32 *) rt);
}