library(tidyverse)
library(extrafont)
library(ggpattern)
library(tikzDevice)
library(xtable)
library(gtable)
library(grid)

source("R/utils.R")

cycles <- function() {
  data <- full_join(
    read_csv("data/diospyros.csv"),
    read_csv("data/est_cycles.csv")
  )

  to_face <- function(sat) {
    map_chr(sat, function(x) {
      if (x == "yes") {
        "black"
      } else {
        "red"
      }  
    })
  }

  faces <- data %>%
    filter(kernel == "dios") %>%
    select(saturated) %>%
    mutate(
      bold=to_face(saturated)
    )

  data <- data %>%
    select(benchmark, params, kernel, cycles) %>%
    group_by(benchmark) %>%
    pivot_wider(
      names_from=kernel,
      values_from=cycles
    ) %>%
    mutate(
      benchmark=recode(
        benchmark,
        "qr-decomp"="QrD",
        "2d-conv"="2DConv",
        "mat-mul"="Matrix Mul",
        "q-prod"="QP"
      ),
      key = str_c(benchmark, params),
      name = rewrite_params(params),
      norm = naive.fixed,
      compgen = norm / compgen,
      dios = norm / dios,
      nature = norm / nature,
      naive.fixed = norm / naive.fixed,
      naive.clang = norm / naive.clang,
      ) %>%
    pivot_longer(
      cols = c("naive.clang", "nature", "dios", "compgen"),
      names_to = "kernel",
      values_to = "cycles"
    ) %>%
    select(name, kernel, benchmark, params, key, cycles)

  names <- (data %>% filter(kernel == "dios"))$name

  p <- data %>%
    group_by(benchmark) %>%
    ggplot(aes(
      ## x=factor(name, levels=unique(name)),
      ## y=cycles,
      xmin=as.numeric(factor(key, levels=unique(key))) - 0.315,
      xmax=as.numeric(factor(key, levels=unique(key))) + 0.315,
      ymin=0,
      ymax=cycles,
      fill=factor(kernel, levels=unique(kernel)),
      )) +
    facet_grid(
      ~benchmark,
      switch="x",
      scales="free_x",
      space="free_x",
      ) +
    geom_rect(
      position="dodge",
      color="black",
      linewidth=0.3
    ) +
    labs(y="Speed up over Clang", fill="Compiler") +
    scale_x_continuous(
      label=names,
      breaks=1:length(names)
    ) +
    scale_y_continuous(
      trans="log2",
      breaks=c(1, 4, 16),
      labels=c("1$\\times$", "4$\\times$", "16$\\times$"),
      ) +
    scale_fill_manual(
      values=c(
        rgb(240,249,232,maxColorValue = 255),
        rgb(186,228,188,maxColorValue = 255),
        rgb(67,162,202,maxColorValue = 255),
        rgb(8,104,172,maxColorValue = 255)
      ),
      labels=c(
        ## "Naive",
        "Clang (auto-vectorized)",
        "Nature",
        "Diospyros",
        "Isaria"
      )
    ) +
    theme_classic() +
    theme(
      axis.title.x = element_blank(),
      axis.title.y = element_text(size=8, face="bold"),

      axis.text.x = element_text(size=8, color="black"),
      axis.text.y = element_text(size=8, color="black", margin=margin(0, 0, 0, 0)),

      legend.position = "top",
      legend.background = element_blank(),
      legend.text = element_text(size=8, face="bold"),
      legend.title = element_blank(),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.margin = margin(0, 0, 2, 0),

      panel.background = element_blank(),
      panel.spacing.x = unit(0.4, "lines"),

      strip.placement = "outside",
      strip.background = element_blank(),
      strip.text = element_text(size=8, face="bold", margin=margin(0, 0, 4, 0)),

      plot.margin = margin(0, 0, 0, 0)
    )

  pb <- ggplot_build(p)
  pg <- ggplot_gtable(pb)

  data2npc <- function(x, panel = 1L, axis = "x") {
    range <- pb$layout$panel_params[[panel]][[paste0(axis,".range")]]
    scales::rescale(c(range, x), c(0,1))[-c(1,2)]
  }

  major_grid <- sapply(c(-1, 0, 1, 2, 3, 4, 5), data2npc, axis="y")

  pg <- gtable_add_grob(
    pg,
    segmentsGrob(x0=0, x1=2, y0=major_grid, y1=major_grid,
                 gp=gpar(col=c("gray", "black", "gray", "gray", "gray", "gray", "gray"),
                         lwd=c(0.9, 1.0, 0.1, 0.9, 0.1, 0.9, 0.1))),
    t=7, b=9,
    l=5, r=11,
    z=0
  )

  grid.newpage()
  grid.draw(pg)
}
