#!/usr/bin/env Rscript

library(conflicted)
library(tidyverse)
conflict_prefer("filter", "dplyr")
conflict_prefer("lag", "dplyr")

library(extrafont)
library(ggpattern)
library(tikzDevice)
library(xtable)
library(gtable)
library(grid)

source("R/cycles.R")
source("R/compilation.R")
source("R/pruning.R")
source("R/ruleset_ablation.R")
source("R/rule_distribution.R")
source("R/alpha_beta.R")
source("R/instruction_ablation.R")

args <- commandArgs(trailingOnly=T)

if (length(args) != 1) {
  stop("Usage: ./generate.R <figure_name>")
}

fig_name <- args[1]
filename = str_c(fig_name, ".tex")

# cycle performance
if (fig_name == "cycle_performance") {
  tikz(filename=filename, width=6.85, height=2.85, standAlone=T)
  cycles()
  dev.off()

# compile time
} else if (fig_name == "compile_time") {
  tikz(filename=filename, width=3.3, height=2, standAlone=T)
  compilation()
  dev.off()

# pruning
} else if (fig_name == "pruning") {
  tikz(filename=filename, width=3.3, height=2, standAlone=T)
  p <- pruning()
  print(p)
  dev.off()

# ruleset ablation
} else if (fig_name == "ruleset_ablation") {
  tikz(filename=filename, width=3.3, height=2, standAlone=T)
  p <- ruleset_ablation()
  print(p)
  dev.off()
  
# rule distribution
} else if (fig_name == "ruleset_distribution") {
  tikz(filename=filename, width=3.3, height=2, standAlone=T)
  alpha_v <- 15
  beta_v <- 12
  p <- rule_distribution(alpha=alpha_v, beta=beta_v)
  print(p)
  dev.off()

# alpha beta
} else if (fig_name == "alpha_beta") {
  tikz(filename=filename, width=3.3, height=2, standAlone=T)
  alpha_v <- 15
  beta_v <- 12
  p <- alpha_beta(alpha=alpha_v, beta=beta_v)
  print(p)
  dev.off()

} else if (fig_name == "instruction_ablation") {
  tbl <- print(instruction_ablation(), include.rownames=F) %>%
    str_replace_all(fixed("+0\\%"), "---")

  standalone_tbl <- str_c(
    "\\documentclass[10pt]{article}\n",
    "\\begin{document}\n",
    tbl,
    "\n",
    "\\end{document}\n"
  )
  writeLines(standalone_tbl, "instruction_ablation.tex")

} else {
  print("Unknown figure type.")
  print("Valid figures are:")
  print(" - cycle_performance")
  print(" - compile_time")
  print(" - pruning")
  print(" - ruleset_ablation")
  print(" - ruleset_distribution")
  print(" - alpha_beta")
  print(" - instruction_ablation")
}

system(str_c("pdflatex ", filename))
system("rm *.log *.aux *.tex")
