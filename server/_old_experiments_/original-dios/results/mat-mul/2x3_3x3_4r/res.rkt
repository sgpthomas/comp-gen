(Concat
  (VecMAC
    (VecMAC
      (VecMul
        (LitVec (Get A 1) (Get A 1) (Get A 1) (Get A 4))
        (LitVec (Get B 3) (Get B 4) (Get B 5) (Get B 3)))
      (LitVec (Get A 2) (Get A 2) (Get A 2) (Get A 5))
      (LitVec (Get B 6) (Get B 7) (Get B 8) (Get B 6)))
    (LitVec (Get A 0) (Get A 0) (Get A 0) (Get A 3))
    (LitVec (Get B 0) (Get B 1) (Get B 2) (Get B 0)))
  (VecMAC
    (VecMAC
      (VecMul (Vec (Get A 4) (Get A 4) 1 1) (LitVec (Get B 4) (Get B 5) 0 0))
      (Vec (Get A 5) (Get A 5) 1 1)
      (LitVec (Get B 7) (Get B 8) 0 0))
    (LitVec (Get A 3) (Get A 3) 0 0)
    (LitVec (Get B 1) (Get B 2) 0 0)))
