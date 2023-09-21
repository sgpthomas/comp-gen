(Concat
  (VecAdd
    (VecAdd
      (VecMul
        (Vec (Get aq 3) (Get aq 3) (Get aq 3) 1)
        (LitVec (Get bq 0) (Get bq 1) (Get bq 2) 0))
      (VecMAC
        (VecMul
          (Vec (Get aq 0) (Get aq 1) (Get aq 2) 1)
          (LitVec (Get bq 3) (Get bq 3) (Get bq 3) 0))
        (LitVec (Get aq 1) (Get aq 2) (Get aq 0) (Get aq 3))
        (LitVec (Get bq 2) (Get bq 0) (Get bq 1) (Get bq 3))))
    (VecAdd
      (VecNeg (VecMul (Vec 1 1 1 (Get aq 0)) (LitVec 0 0 0 (Get bq 0))))
      (VecAdd
        (VecNeg (VecMul (Vec 1 1 1 (Get aq 1)) (LitVec 0 0 0 (Get bq 1))))
        (VecNeg
          (VecMul
            (LitVec (Get aq 2) (Get aq 0) (Get aq 1) (Get aq 2))
            (LitVec (Get bq 1) (Get bq 2) (Get bq 0) (Get bq 2)))))))
  (VecAdd
    (LitVec (Get at 0) (Get at 1) (Get at 2) 0)
    (VecAdd
      (LitVec (Get bt 0) (Get bt 1) (Get bt 2) 0)
      (VecMAC
        (VecAdd
          (VecMul
            (Vec (Get aq 1) (Get aq 2) (Get aq 0) 1)
            (VecMul
              (Vec 2 2 2 1)
              (VecAdd
                (VecMul
                  (Vec (Get aq 0) (Get aq 1) (Get aq 2) 1)
                  (LitVec (Get bt 1) (Get bt 2) (Get bt 0) 0))
                (VecNeg
                  (VecMul
                    (Vec (Get aq 1) (Get aq 2) (Get aq 0) 1)
                    (LitVec (Get bt 0) (Get bt 1) (Get bt 2) 0))))))
          (VecNeg
            (VecMul
              (Vec (Get aq 2) (Get aq 0) (Get aq 1) 1)
              (VecMul
                (Vec 2 2 2 1)
                (VecAdd
                  (VecMul
                    (Vec (Get aq 2) (Get aq 0) (Get aq 1) 1)
                    (LitVec (Get bt 0) (Get bt 1) (Get bt 2) 0))
                  (VecNeg
                    (VecMul
                      (Vec (Get aq 0) (Get aq 1) (Get aq 2) 1)
                      (LitVec (Get bt 2) (Get bt 0) (Get bt 1) 0))))))))
        (Vec (Get aq 3) (Get aq 3) (Get aq 3) 1)
        (VecMul
          (Vec 2 2 2 1)
          (VecAdd
            (VecMul
              (Vec (Get aq 1) (Get aq 2) (Get aq 0) 1)
              (LitVec (Get bt 2) (Get bt 0) (Get bt 1) 0))
            (VecNeg
              (VecMul
                (Vec (Get aq 2) (Get aq 0) (Get aq 1) 1)
                (LitVec (Get bt 1) (Get bt 2) (Get bt 0) 0)))))))))
