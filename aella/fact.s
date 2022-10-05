//
// Assembler program to print "Hello World!"
// to stdout.
//
// X0-X2 - parameters to linux function services
// X16 - linux function number
//
.global _start             // Provide program starting address to linker
.align 2

// Setup the parameters to print hello world
// and then call Linux to do it.

_start: mov X19, #4          // Z = 5
	mov X20, #1          // Y = 1

body:
	cbz X19, end
	mov X21, #4
	smull X20, W20, W19
	sub X19, X19, #1
	b body

end:
        mov X0, X20      // exit with 0
        mov X16, #1
        svc 0

string:	.ascii "The number is %d:\n"
