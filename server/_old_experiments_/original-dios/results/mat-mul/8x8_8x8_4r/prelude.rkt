(prog
 (list
  (vec-extern-decl 'A 64 'extern-input-array)
  (vec-extern-decl 'B 64 'extern-input-array)
  (vec-extern-decl 'C 64 'extern-output)
  (vec-const 'Z '#(0.0) 'float)))
