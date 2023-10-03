ruleset_ablation <- function() {
  data <- read_csv("data/ruleset_ablation.csv") %>% select(-index)
  ## noeqsat <- read_csv("data/noeqsat.csv") %>%
  ##   mutate(ruleset=0, noeqsat=cycles) %>%
  ##   select(-c(kernel, correct, cycles, ruleset)) %>%
  ##   filter(benchmark == "2d-conv")
  baseline <- read_csv("data/diospyros.csv") %>%
    filter(kernel == "naive.fixed") %>%
    mutate(baseline=cycles) %>%
    select(benchmark, params, baseline)

  data <- left_join(
    data,
    baseline,
    by=c("benchmark", "params"),
    )

  data <- data %>%
    select(benchmark, params, exp, ruleset, cycles, cost, baseline) %>%
    mutate(
      name=params %>% str_replace_all(c(
        "2x2"="2$^2$",
        "3x3"="3$^2$",
        "4x4"="4$^2$",
        "8x8"="8$^2$",
        "10x10"="10$^2$",
        "16x16"="16$^2$",
        "18x18"="18$^2$",
        "_"="\n"
      )),
      ) %>%
    group_by(params) %>%
    filter(ruleset > 0) %>%
    filter(ruleset != 43200) %>%
    filter(ruleset != 86400) %>%
    mutate(
      # calculate speedup against the second item in every group
      ## across(cycles:cost, ~ .[1] / .)
      cycles=baseline / cycles
      ## across(cycles:cost, ~ .[3] / .)
    )

  data %>%
    ggplot(aes(
      ## x=names,
      ## y=cycles,
      xmin=as.numeric(factor(name, levels=unique(name))) - 0.35,
      xmax=as.numeric(factor(name, levels=unique(name))) + 0.35,
      ymin=0, ymax=cycles,
      fill=factor(ruleset)
    )) +
    geom_hline(
        yintercept = 1,
        color = "black",
        linewidth=0.35
    ) +
    geom_rect(
      position="dodge",
      color="black",
      linewidth=0.3
    ) +
    scale_x_continuous(
      label=unique(data$name),
      breaks=1:length(unique(data$name))
    ) +
    scale_y_continuous(
      labels=c("0.25$\\times$", "1$\\times$", "4$\\times$", "16$\\times$"),
      breaks=c(0.25, 1, 4, 16),
      limits=c(0.1, 32),
      expand=c(0, 0),
      trans="log2"
    ) +
    scale_fill_brewer(
      palette = "YlOrBr"
    ) +
    labs(fill="Timeout", y="Speedup over Clang", x="2DConv") +
    theme_classic() +
    theme(
      axis.title.x = element_text(size=8, face="bold"),
      axis.title.y = element_text(size=8, face="bold"),

      axis.text.x = element_text(size=8, color="black"),
      axis.text.y = element_text(size=8, color="black"),

      legend.position = "top",
      legend.background = element_blank(),
      legend.text = element_text(size=7),
      legend.title = element_text(size=7, face="bold"),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.margin = margin(0, 0, 2, 0),

      panel.grid.major.y = element_line(color="grey", linewidth=0.4),
      panel.grid.minor.y = element_line(color="grey", linewidth=0.1),

      plot.margin = margin(0, 0, 0, 0)
    )
}
