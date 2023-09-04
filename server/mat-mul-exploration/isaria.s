	.text
	.file	"isaria.c"
	.literal_position
	.literal	.LCPI0_0, .L_MergedGlobals
	.globl	_Z6kernelPfS_S_
	.p2align	2
	.type	_Z6kernelPfS_S_,@function
_Z6kernelPfS_S_:
	entry	a1,32		#[0]
 {	# format FF
	l32r	a5,.LCPI0_0		#[1] @57,37
	movi	a6,16		#[1]
 }
 {	# format FF
	pdx_la_v_pp	u0,a2		#[2] @40,13
	pdx_la_v_pp	u1,a3		#[2] @43,13
 }
 {	# format FF
	pdx_lav_v_xp	v0,u0,a2,a6		#[3] @47,3
	pdx_lav_v_xp	v2,u1,a3,a6		#[3] @51,3
 }
 {	# format FF
	pdx_lav_v_xp	v1,u0,a2,a6		#[4] @49,3
	pdx_lav_v_xp	v4,u1,a3,a6		#[4] @53,3
 }
 {	# format FF
	pdx_lv_v_i	v3,a5,0		#[5] @57,37
	pdx_lv_v_i	v5,a5,16		#[5] @59,37
 }
 {	# format FB
	pdx_lav_v_xp	v6,u1,a3,a6		#[6] @55,3
	pdx_lv_v_i	v7,a5,96		#[6] @74,39
	pdx_sel_mx32	v3,v1,v0,v3		#[6] @57,9
	nop	
 }
 {	# format FB
	pdx_lv_v_i	v8,a5,112		#[7] @76,32
	pdx_lv_v_i	v9,a5,32		#[7] @62,37
	pdx_sel_mx32	v5,v4,v2,v5		#[7] @59,9
	nop	
 }
 {	# format FB
	pdx_lv_v_i	v10,a5,48		#[8] @64,38
	pdx_lv_v_i	v22,a5,128		#[8] @79,32
	pdx_sel_mx32	v7,v6,v4,v7		#[8] @74,10
	pdx_shfl_mx32	v8,v1,v8		#[8] @76,10
 }
 {	# format FB
	pdx_lv_v_i	v23,a5,144		#[9] @81,32
	pdx_lv_v_i	v24,a5,64		#[9] @68,32
	pdx_sel_mx32	v21,v6,v4,v10		#[9] @64,9
	pdx_shfl_mx32	v6,v4,v22		#[9] @79,10
 }
 {	# format FB
	pdx_lv_v_i	v25,a5,80		#[10] @70,38
	pdx_lv_v_i	v27,a5,160		#[10] @85,32
	pdx_sel_mx32	v9,v1,v0,v9		#[10] @62,9
	pdx_mul_mxf32	v3,v3,v5		#[10] @60,21
 }
 {	# format FB
	pdx_lv_v_i	v28,a5,176		#[11] @87,32
	pdx_z_align	u0		#[11] @45,20
	pdx_shfl_mx32	v1,v1,v23		#[11] @81,10
	pdx_mul_mxf32	v7,v7,v8		#[11] @77,22
 }
 {	# format FB
	movi	a2,8		#[12]
	nop	
	pdx_sel_mx32	v4,v4,v2,v25		#[12] @70,10
	pdx_shfl_mx32	v26,v0,v24		#[12] @68,10
 }
 {	# format FB
	nop	
	nop	
	pdx_shfl_mx32	v0,v0,v27		#[13] @85,10
	pdx_shfl_mx32	v29,v2,v28		#[13] @87,10
 }
 {	# format FH
	nop	
	nop	
	nop	
	pdx_mula_mxf32	v3,v9,v21		#[14] @66,3
 }
 {	# format FH
	nop	
	nop	
	nop	
	pdx_mula_mxf32	v7,v6,v1		#[15] @83,3
 }
 {	# format FH
	nop	
	nop	
	nop	
	pdx_mula_mxf32	v3,v26,v4		#[18] interlock @72,3
 }
 {	# format FH
	nop	
	nop	
	nop	
	pdx_mula_mxf32	v7,v0,v29		#[19] @89,3
 }
 {	# format FF
	pdx_sav_v_xp	v3,u0,a4,a6		#[24] interlock @90,3
	nop	
 }
 {	# format FF
	pdx_sav_v_xp	v7,u0,a4,a2		#[25] @91,3
	nop	
 }
 {	# format FF
	pdx_sapos_fp	u0,a4		#[26] @92,3
	nop	
 }
	retw.n			#[27] @93,1
.Lfunc_end0:
	.size	_Z6kernelPfS_S_, .Lfunc_end0-_Z6kernelPfS_S_

	.type	Z,@object
	.section	.dram0.data
	.globl	Z
	.p2align	4
Z:
	.space	16
	.size	Z, 16

	.type	v_0,@object
	.globl	v_0
	.p2align	4
v_0:
	.space	16
	.size	v_0, 16

	.type	v_1,@object
	.globl	v_1
	.p2align	4
v_1:
	.long	1
	.long	0
	.long	1
	.long	4
	.size	v_1, 16

	.type	v_3,@object
	.globl	v_3
	.p2align	4
v_3:
	.long	3
	.long	1
	.long	5
	.long	3
	.size	v_3, 16

	.type	v_6,@object
	.globl	v_6
	.p2align	4
v_6:
	.long	2
	.long	1
	.long	2
	.long	5
	.size	v_6, 16

	.type	v_8,@object
	.globl	v_8
	.p2align	4
v_8:
	.long	6
	.long	4
	.long	8
	.long	6
	.size	v_8, 16

	.type	v_11,@object
	.globl	v_11
	.p2align	4
v_11:
	.long	0
	.long	2
	.long	0
	.long	3
	.size	v_11, 16

	.type	v_13,@object
	.globl	v_13
	.p2align	4
v_13:
	.long	0
	.long	7
	.long	2
	.long	0
	.size	v_13, 16

	.type	v_16,@object
	.globl	v_16
	.p2align	4
v_16:
	.long	7
	.long	8
	.long	7
	.long	7
	.size	v_16, 16

	.type	v_18,@object
	.globl	v_18
	.p2align	4
v_18:
	.long	5
	.long	5
	.long	5
	.long	5
	.size	v_18, 16

	.type	v_21,@object
	.globl	v_21
	.p2align	4
v_21:
	.long	4
	.long	5
	.long	4
	.long	4
	.size	v_21, 16

	.type	v_23,@object
	.globl	v_23
	.p2align	4
v_23:
	.long	4
	.long	4
	.long	4
	.long	4
	.size	v_23, 16

	.type	v_26,@object
	.globl	v_26
	.p2align	4
v_26:
	.long	3
	.long	3
	.long	3
	.long	3
	.size	v_26, 16

	.type	v_28,@object
	.globl	v_28
	.p2align	4
v_28:
	.long	1
	.long	2
	.long	1
	.long	1
	.size	v_28, 16

	.type	.L_MergedGlobals,@object
	.p2align	4
.L_MergedGlobals:
	.long	1
	.long	0
	.long	1
	.long	4
	.long	3
	.long	1
	.long	5
	.long	3
	.long	2
	.long	1
	.long	2
	.long	5
	.long	2
	.long	0
	.long	4
	.long	2
	.long	0
	.long	2
	.long	0
	.long	3
	.long	0
	.long	7
	.long	2
	.long	0
	.long	3
	.long	4
	.long	3
	.long	3
	.long	1
	.long	1
	.long	1
	.long	1
	.long	0
	.long	1
	.long	0
	.long	0
	.space	16
	.long	3
	.long	3
	.long	3
	.long	3
	.long	1
	.long	2
	.long	1
	.long	1
	.size	.L_MergedGlobals, 192

	.globl	v_1_0
.set v_1_0, .L_MergedGlobals
	.size	v_1_0, 16
	.globl	v_3_0
.set v_3_0, .L_MergedGlobals+16
	.size	v_3_0, 16
	.globl	v_6_0
.set v_6_0, .L_MergedGlobals+32
	.size	v_6_0, 16
	.globl	v_8_0
.set v_8_0, .L_MergedGlobals+48
	.size	v_8_0, 16
	.globl	v_11_0
.set v_11_0, .L_MergedGlobals+64
	.size	v_11_0, 16
	.globl	v_13_0
.set v_13_0, .L_MergedGlobals+80
	.size	v_13_0, 16
	.globl	v_16_0
.set v_16_0, .L_MergedGlobals+96
	.size	v_16_0, 16
	.globl	v_18_0
.set v_18_0, .L_MergedGlobals+112
	.size	v_18_0, 16
	.globl	v_21_0
.set v_21_0, .L_MergedGlobals+128
	.size	v_21_0, 16
	.globl	v_23_0
.set v_23_0, .L_MergedGlobals+144
	.size	v_23_0, 16
	.globl	v_26_0
.set v_26_0, .L_MergedGlobals+160
	.size	v_26_0, 16
	.globl	v_28_0
.set v_28_0, .L_MergedGlobals+176
	.size	v_28_0, 16
	.ident	"XtensaTools-14.08 clang version 10.0.1 "
	.section	".note.GNU-stack","",@progbits
