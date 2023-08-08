source("R/utils.R")

pruning <- function() {
  data <- read_csv("data/pruning.csv")
  killed_height <- 4500
  data <- data %>%
    select(params, pruning, cycles, compile_time) %>%
    pivot_wider(names_from=pruning, values_from=c(cycles, compile_time)) %>%
    mutate(
      cycles=replace_na(cycles_FALSE / cycles_TRUE, Inf),
      time=if_else(is.infinite(cycles), Inf, compile_time_FALSE / compile_time_TRUE),
      ## cycles=if_else(killed, killed_height, cycles),
      params=rewrite_params(params)
    ) %>%
    select(params, cycles, time) %>%
    
    pivot_longer(
      cols = -c(params),
      names_to="name",
      values_to="speedup"
    )

  data %>%
    ggplot(aes(
      xmin=as.numeric(factor(params, levels=unique(params))) - 0.25,
      xmax=as.numeric(factor(params, levels=unique(params))) + 0.25,
      ymin=0,
      ymax=speedup,
      fill=name,
      pattern=is.infinite(speedup)
    )) +
    geom_hline(
      yintercept=1,
      color="black",
      linewidth=0.35
    ) +
    geom_rect_pattern(
      position="dodge",
      color="black",
      pattern_fill="black",
      pattern_linetype=0,
      pattern_alpha=0.35,
      pattern_spacing=0.05,
      pattern_density=0.30,
      linewidth=0.3
    ) +
    scale_fill_manual(
      values=c("#a8ddb5", "#43a2ca"),
      labels=c("Kernel Performance", "Compile Time")
    ) +
    scale_pattern_manual(
      values=c("none", "stripe")
    ) +
    scale_x_continuous(
      label=unique(data$params),
      breaks=1:length(unique(data$params))
    ) +
    scale_y_continuous(
      trans="log2",
      labels=c("$0.25\\times$", "$1\\times$", "$4\\times$", "$16\\times$"),
      breaks=c(0.25, 1, 4, 16),
      ## expand=c(0, 0)
    ) +
    labs(
      x="2DConv",
      y="Speedup",
      fill="Pruning"
    ) +
    guides(pattern="none", fill=guide_legend(override.aes = list(pattern="none"))) +
    theme_classic() +
    theme(
      axis.title.x = element_text(size=8, face="bold"),
      axis.title.y = element_text(size=8, face="bold"),

      axis.text.x = element_text(size=8, color="black"),
      axis.text.y = element_text(size=8, color="black"),

      legend.position = "top",
      legend.background = element_blank(),
      legend.text = element_text(size=7, face="bold"),
      legend.title = element_blank(),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.margin = margin(0, 0, 4, 0),

      panel.grid.major.y = element_line(color="grey", linewidth=0.4),
      panel.grid.minor.y = element_line(color="grey", linewidth=0.1),

      plot.margin = margin(1, 0, 1, 0)
      )
}
