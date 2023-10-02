library(tidyverse)
library(extrafont)
library(ggpattern)
library(tikzDevice)
library(xtable)
library(gtable)
library(grid)

source("R/utils.R")

alpha_beta <- function(alpha_v=15, beta_v=12) {
  data <- read_csv("data/alpha_beta.csv")

  data %>%
    ggplot() +
    geom_tile(
      aes(x = as.factor(beta * 2), y = as.factor(alpha), fill = cycles),
      color="black"
    ) +
    geom_tile(
      aes(x = as.factor(beta_v), y = as.factor(alpha_v)),
      fill = NA,
      color="salmon", lwd=0.7
    ) +
    labs(
      title="Estimated Cycles for 2d-conv $16^2 \\times 4^2$",
      x="$\\beta$ (aggregate cost)",
      y="$\\alpha$ (cost differential)"
    ) +
    scale_fill_gradient(na.value = "light gray") +
    guides(
      fill = guide_colorbar(
        barwidth=0.3,
        barheight=8,
        ticks=F
      ),
      color = "none"
    ) +
    theme(
      axis.title.x = element_text(size=7, face="bold"),
      axis.title.y = element_text(size=7, face="bold"),
      
      axis.text.x = element_text(size=6, color="black"),
      axis.text.y = element_text(size=7, color="black"),

      legend.position = "right",
      legend.background = element_blank(),
      legend.text = element_text(size=7),
      legend.title = element_blank(),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.spacing.x = unit(0.2, "lines"),
      legend.margin = margin(0, 0, 5, 2),

      panel.background = element_blank(),
      panel.spacing.x = unit(0.4, "lines"),

      strip.background = element_blank(),

      plot.title = element_text(size=8, face="bold", hjust=0.5),
      plot.margin = margin(1, 0, 1, 0)
    )
}
