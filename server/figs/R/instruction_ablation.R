library(tidyverse)
library(extrafont)
library(ggpattern)
library(tikzDevice)
library(xtable)
library(gtable)
library(grid)

source("R/utils.R")

instruction_ablation <- function() {
  data <- read_csv(
    "data/instruction.csv",
    show_col_types=F,
    progress=F,
    col_names=T
  ) %>%
    filter(benchmark == "qr-decomp") %>%
    mutate(
      muls=if_else(str_detect(rules, "muls"), "VecMulSub", "No VecMulSub"),
      sqrtsgn=if_else(str_detect(rules, "sqrtsgn"), "VecSqrtSgn", "No VecSqrtSgn"),
      speedup=(1198.0 / cycles) * 100,
      show=if_else(speedup >= 100,
                   str_c("+", round(speedup-100, 1), "%"),
                   str_c("-", 100-speedup, "%"))
    ) %>%
    select(muls, sqrtsgn, show) %>%
    pivot_wider(names_from=muls, values_from=show) %>%
    mutate(` `=sqrtsgn)

  xtable(
    data,
    caption=str_c(
      "Cycle estimates for QR-Decomp for all",
      " combinations of including MAC and MULS instructions."
    ),
    label="tab:instruction"
  )
}
