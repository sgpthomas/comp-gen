	.text
	.file	"autovec.c"
	.literal_position
	.globl	_Z31naive_matrix_multiply_hard_sizePfS_S_
	.p2align	2
	.type	_Z31naive_matrix_multiply_hard_sizePfS_S_,@function
_Z31naive_matrix_multiply_hard_sizePfS_S_:
	entry	a1,32		#[0]
 {	# format FF
	pdx_ls_32_i	v0,a2,4		#[1] @19,30
	pdx_ls_32_i	v1,a3,12		#[1] @19,50
 }
 {	# format FB
	pdx_ls_32_i	v4,a3,16		#[2] @19,50
	pdx_ls_32_i	v2,a2,16		#[2] @19,30
	nop	
	mul.s	v3,v0,v1		#[2] @19,48
 }
 {	# format FB
	pdx_ls_32_i	v6,a3,20		#[3] @19,50
	pdx_ls_32_i	v7,a2,0		#[3] @19,30
	nop	
	mul.s	v5,v0,v4		#[3] @19,48
 }
 {	# format FB
	pdx_ls_32_i	v8,a3,0		#[4] @19,50
	pdx_ls_32_i	v22,a2,12		#[4] @19,30
	nop	
	mul.s	v1,v2,v1		#[4] @19,48
 }
 {	# format FH
	pdx_ls_32_i	v23,a3,4		#[5] @19,50
	nop	
	nop	
	mul.s	v4,v2,v4		#[5] @19,48
 }
 {	# format FH
	pdx_ls_32_i	v24,a3,8		#[6] @19,50
	nop	
	nop	
	mul.s	v0,v0,v6		#[6] @19,48
 }
 {	# format FH
	pdx_ls_32_i	v27,a2,20		#[7] @19,30
	nop	
	nop	
	mul.s	v2,v2,v6		#[7] @19,48
 }
 {	# format FH
	pdx_ls_32_i	v29,a3,32		#[8] @19,50
	nop	
	nop	
	madd.s	v3,v7,v8		#[8] @19,27
 }
 {	# format FH
	pdx_ls_32_i	v28,a3,28		#[9] @19,50
	nop	
	nop	
	madd.s	v1,v22,v8		#[9] @19,27
 }
 {	# format FH
	pdx_ls_32_i	v26,a3,24		#[10] @19,50
	nop	
	nop	
	madd.s	v5,v7,v23		#[10] @19,27
 }
 {	# format FH
	pdx_ls_32_i	v25,a2,8		#[11] @19,30
	nop	
	nop	
	madd.s	v4,v22,v23		#[11] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v0,v7,v24		#[12] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v2,v22,v24		#[13] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v3,v25,v26		#[14] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v1,v27,v26		#[15] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v5,v25,v28		#[16] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v4,v27,v28		#[17] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v0,v25,v29		#[18] @19,27
 }
 {	# format FH
	nop	
	nop	
	nop	
	madd.s	v2,v27,v29		#[19] @19,27
 }
 {	# format FF
	pdx_ss_32_i	v3,a4,0		#[20] @19,27
	nop	
 }
 {	# format FF
	pdx_ss_32_i	v1,a4,12		#[21] @19,27
	nop	
 }
 {	# format FF
	pdx_ss_32_i	v5,a4,4		#[22] @19,27
	nop	
 }
 {	# format FF
	pdx_ss_32_i	v4,a4,16		#[23] @19,27
	nop	
 }
 {	# format FF
	pdx_ss_32_i	v0,a4,8		#[24] @19,27
	nop	
 }
 {	# format FF
	pdx_ss_32_i	v2,a4,20		#[25] @19,27
	nop	
 }
	retw.n			#[26] @23,1
.Lfunc_end0:
	.size	_Z31naive_matrix_multiply_hard_sizePfS_S_, .Lfunc_end0-_Z31naive_matrix_multiply_hard_sizePfS_S_

	.ident	"XtensaTools-14.08 clang version 10.0.1 "
	.section	".note.GNU-stack","",@progbits
