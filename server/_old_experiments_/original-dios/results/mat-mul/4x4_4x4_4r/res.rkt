(Concat
  (VecMAC
    (VecAdd
      (VecMul
        (LitVec (Get A 1) (Get A 1) (Get A 1) (Get A 1))
        (LitVec (Get B 4) (Get B 5) (Get B 6) (Get B 7)))
      (VecMAC
        (VecMul
          (LitVec (Get A 2) (Get A 2) (Get A 2) (Get A 2))
          (LitVec (Get B 8) (Get B 9) (Get B 10) (Get B 11)))
        (LitVec (Get A 3) (Get A 3) (Get A 3) (Get A 3))
        (LitVec (Get B 12) (Get B 13) (Get B 14) (Get B 15))))
    (LitVec (Get A 0) (Get A 0) (Get A 0) (Get A 0))
    (LitVec (Get B 0) (Get B 1) (Get B 2) (Get B 3)))
  (Concat
    (VecMAC
      (VecAdd
        (VecMul
          (LitVec (Get A 5) (Get A 5) (Get A 5) (Get A 5))
          (LitVec (Get B 4) (Get B 5) (Get B 6) (Get B 7)))
        (VecMAC
          (VecMul
            (LitVec (Get A 6) (Get A 6) (Get A 6) (Get A 6))
            (LitVec (Get B 8) (Get B 9) (Get B 10) (Get B 11)))
          (LitVec (Get A 7) (Get A 7) (Get A 7) (Get A 7))
          (LitVec (Get B 12) (Get B 13) (Get B 14) (Get B 15))))
      (LitVec (Get A 4) (Get A 4) (Get A 4) (Get A 4))
      (LitVec (Get B 0) (Get B 1) (Get B 2) (Get B 3)))
    (Concat
      (VecMAC
        (VecAdd
          (VecMul
            (LitVec (Get A 9) (Get A 9) (Get A 9) (Get A 9))
            (LitVec (Get B 4) (Get B 5) (Get B 6) (Get B 7)))
          (VecMAC
            (VecMul
              (LitVec (Get A 10) (Get A 10) (Get A 10) (Get A 10))
              (LitVec (Get B 8) (Get B 9) (Get B 10) (Get B 11)))
            (LitVec (Get A 11) (Get A 11) (Get A 11) (Get A 11))
            (LitVec (Get B 12) (Get B 13) (Get B 14) (Get B 15))))
        (LitVec (Get A 8) (Get A 8) (Get A 8) (Get A 8))
        (LitVec (Get B 0) (Get B 1) (Get B 2) (Get B 3)))
      (VecMAC
        (VecAdd
          (VecMul
            (LitVec (Get A 13) (Get A 13) (Get A 13) (Get A 13))
            (LitVec (Get B 4) (Get B 5) (Get B 6) (Get B 7)))
          (VecMAC
            (VecMul
              (LitVec (Get A 14) (Get A 14) (Get A 14) (Get A 14))
              (LitVec (Get B 8) (Get B 9) (Get B 10) (Get B 11)))
            (LitVec (Get A 15) (Get A 15) (Get A 15) (Get A 15))
            (LitVec (Get B 12) (Get B 13) (Get B 14) (Get B 15))))
        (LitVec (Get A 12) (Get A 12) (Get A 12) (Get A 12))
        (LitVec (Get B 0) (Get B 1) (Get B 2) (Get B 3))))))
