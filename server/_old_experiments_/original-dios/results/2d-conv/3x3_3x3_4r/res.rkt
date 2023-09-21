(Concat
  (VecMAC
    (VecMAC
      (VecMul (Vec 1 1 (Get I 1) 1) (LitVec 0 0 (Get F 1) 0))
      (Vec 1 (Get I 0) (Get I 2) (Get I 1))
      (LitVec 0 (Get F 1) (Get F 0) (Get F 2)))
    (LitVec (Get I 0) (Get I 1) (Get I 0) (Get I 2))
    (LitVec (Get F 0) (Get F 0) (Get F 2) (Get F 1)))
  (Concat
    (VecMAC
      (VecAdd
        (VecMul (Vec 1 1 1 (Get I 1)) (LitVec 0 0 0 (Get F 4)))
        (VecAdd
          (VecMul (Vec 1 1 1 (Get I 2)) (LitVec 0 0 0 (Get F 3)))
          (VecAdd
            (VecMul (Vec 1 1 (Get I 1) (Get I 3)) (LitVec 0 0 (Get F 3) (Get F 2)))
            (VecMAC
              (VecMul (Vec 1 1 (Get I 3) (Get I 4)) (LitVec 0 0 (Get F 1) (Get F 1)))
              (Vec 1 (Get I 0) (Get I 4) (Get I 5))
              (LitVec 0 (Get F 3) (Get F 0) (Get F 0))))))
      (LitVec (Get I 2) (Get I 3) (Get I 0) (Get I 0))
      (LitVec (Get F 2) (Get F 0) (Get F 4) (Get F 5)))
    (Concat
      (VecMAC
        (VecAdd
          (VecMul (Vec 1 1 1 (Get I 1)) (LitVec 0 0 0 (Get F 6)))
          (VecAdd
            (VecMul (Vec 1 1 1 (Get I 3)) (LitVec 0 0 0 (Get F 4)))
            (VecAdd
              (VecMul (Vec (Get I 2) 1 1 (Get I 4)) (LitVec (Get F 4) 0 0 (Get F 3)))
              (VecMAC
                (VecMul
                  (Vec (Get I 4) 1 (Get I 3) (Get I 6))
                  (LitVec (Get F 2) 0 (Get F 3) (Get F 1)))
                (LitVec (Get I 5) (Get I 2) (Get I 6) (Get I 7))
                (LitVec (Get F 1) (Get F 5) (Get F 0) (Get F 0))))))
        (LitVec (Get I 1) (Get I 5) (Get I 0) (Get I 0))
        (LitVec (Get F 5) (Get F 2) (Get F 6) (Get F 7)))
      (Concat
        (VecMAC
          (VecAdd
            (VecMul (Vec (Get I 1) 1 1 1) (LitVec (Get F 7) 0 0 0))
            (VecAdd
              (VecMul (Vec (Get I 2) 1 1 1) (LitVec (Get F 6) 0 0 0))
              (VecAdd
                (VecMul (Vec (Get I 3) 1 1 1) (LitVec (Get F 5) 0 0 0))
                (VecAdd
                  (VecMul (Vec (Get I 4) (Get I 2) 1 1) (LitVec (Get F 4) (Get F 7) 0 0))
                  (VecAdd
                    (VecMul (Vec (Get I 5) (Get I 4) 1 1) (LitVec (Get F 3) (Get F 5) 0 0))
                    (VecAdd
                      (VecMul (Vec (Get I 6) (Get I 5) 1 1) (LitVec (Get F 2) (Get F 4) 0 0))
                      (VecMAC
                        (VecMul
                          (Vec (Get I 7) (Get I 7) (Get I 5) 1)
                          (LitVec (Get F 1) (Get F 2) (Get F 5) 0))
                        (LitVec (Get I 8) (Get I 8) (Get I 8) (Get I 3))
                        (LitVec (Get F 0) (Get F 1) (Get F 2) (Get F 6)))))))))
          (LitVec (Get I 0) (Get I 1) (Get I 2) (Get I 6))
          (LitVec (Get F 8) (Get F 8) (Get F 8) (Get F 3)))
        (Concat
          (VecMAC
            (VecAdd
              (VecMul (Vec 1 (Get I 4) 1 1) (LitVec 0 (Get F 7) 0 0))
              (VecAdd
                (VecMul (Vec 1 (Get I 5) (Get I 5) 1) (LitVec 0 (Get F 6) (Get F 7) 0))
                (VecAdd
                  (VecMul (Vec (Get I 4) (Get I 6) 1 1) (LitVec (Get F 6) (Get F 5) 0 0))
                  (VecMAC
                    (VecMul
                      (Vec (Get I 6) (Get I 7) (Get I 7) 1)
                      (LitVec (Get F 4) (Get F 4) (Get F 5) 0))
                    (LitVec (Get I 7) (Get I 8) (Get I 8) (Get I 5))
                    (LitVec (Get F 3) (Get F 3) (Get F 4) (Get F 8))))))
            (LitVec (Get I 3) (Get I 3) (Get I 4) (Get I 8))
            (LitVec (Get F 7) (Get F 8) (Get F 8) (Get F 5)))
          (Concat
            (VecMAC
              (VecMAC
                (VecMul (Vec 1 1 (Get I 7) 1) (LitVec 0 0 (Get F 7) 0))
                (Vec 1 (Get I 6) (Get I 8) (Get I 7))
                (LitVec 0 (Get F 7) (Get F 6) (Get F 8)))
              (LitVec (Get I 6) (Get I 7) (Get I 6) (Get I 8))
              (LitVec (Get F 6) (Get F 6) (Get F 8) (Get F 7)))
            (VecMul (LitVec (Get I 8) 0 0 0) (LitVec (Get F 8) 0 0 0))))))))
