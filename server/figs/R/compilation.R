compilation <- function() {
  data <- full_join(
    read_csv("data/diospyros.csv") %>% mutate(compile_time=eqsat_time),
    read_csv("data/est_cycles.csv")
  )

  # fix the order of params cat var
  ## data$params <- factor(data$params, levels=unique(data$params))

  data <- data %>%
    ##  %>%
    ## filter(greedy == "True") %>%
    ## filter(benchmark == "2d-conv" | benchmark == "mat-mul") %>%
    ## filter(params != "3x5_3x3") %>%
    ## filter(params != "3x3_2x2") %>%
    ## filter(params != "2x3_3x3") %>%
    select(benchmark, params, kernel, compile_time) %>%
    pivot_wider(
      names_from=kernel,
      values_from=compile_time
    ) %>%
    group_by(benchmark) %>%
    mutate(
      benchmark=recode(benchmark,
                       "2d-conv"="2DConv",
                       "mat-mul"="Matrix Mul",
                       "qr-decomp"="QrD",
                       "q-prod"="Q"),
      ## name=str_replace_all(params, "_", "\n")
      ## name=if_else(str_detect(params, "x"), row_number(), params),
      name=row_number()
    ) %>%
    print(n=100) %>%
    pivot_longer(
      cols = c("dios", "compgen"),
      names_to = "kernel",
      values_to = "compile_time"
    )

  p <- data %>%
    ggplot(aes(
      x=factor(name, levels=unique(name)),
      y=compile_time,
      fill=factor(kernel, levels=unique(kernel))
    )) +
    facet_grid(
      ~benchmark,
      switch="x",
      scales = "free_x", space="free_x"
    ) +
    geom_col(
      position="dodge",
      color="black",
      width=0.75,
      linewidth=0.3
    ) +
    ## geom_hline(
    ##   yintercept=180,
    ## ) +
    ## coord_cartesian(ylim = c(0, 1000)) +
    ## scale_y_log10() +
    scale_fill_manual(
      values=c(
        rgb(67,162,202,maxColorValue = 255),
        rgb(8,104,172,maxColorValue = 255)
      ),
      labels=c("Diospyros", "Isaria")
    ) +
    scale_y_continuous(
      breaks=c(0, 300, 600, 900),
      labels=c("0s", "300s", "600s", "900s"),
      expand=c(0, 0)
    ) +
    labs(y="Compile Time", fill="Compiler") +
    theme_classic() +
    theme(
      axis.title.x = element_blank(),
      axis.title.y = element_text(size=8, face="bold"),

      ## axis.text.x = element_text(size=5, color="black"),
      axis.text.x = element_blank(),
      axis.text.y = element_text(size=8, color="black"),

      legend.position = "top",
      legend.background = element_blank(),
      legend.text = element_text(size=8, face="bold"),
      legend.title = element_blank(),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.margin = margin(0, 0, 2, 0),

      panel.background = element_blank(),
      panel.spacing.x = unit(0.2, "lines"),

      strip.placement = "outside",
      strip.text.x = element_text(size=8, face="bold", margin=margin(0, 0, 4, 0)),
      strip.background = element_blank(),

      plot.margin = margin(0, 0, 0, 0)
    )

  pb <- ggplot_build(p)
  pg <- ggplot_gtable(pb)

  data2npc <- function(x, panel = 1L, axis = "x") {
    range <- pb$layout$panel_params[[panel]][[paste0(axis,".range")]]
    scales::rescale(c(range, x), c(0,1))[-c(1,2)]
  }

  major_grid <- sapply(c(0, 150, 300, 450, 600, 750, 900, 1050), data2npc, axis="y")

  pg <- gtable_add_grob(
    pg,
    segmentsGrob(x0=0, x1=2, y0=major_grid, y1=major_grid,
                 gp=gpar(col=c("gray", "gray", "gray",
                               "gray", "gray", "gray", "gray", "gray"),
                         lwd=c(0.9, 0.1, 0.9, 0.1, 0.9, 0.1, 0.9, 0.1))),
    t=7, b=9,
    l=5, r=11,
    z=0
  )

  grid.newpage()
  grid.draw(pg)
}
