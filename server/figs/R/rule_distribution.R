piecewise <- function(L, H, W, gm=1, inv=F) {
  f    <- function(x) x
  g    <- function(x) (gm * (x - H)) + L + W
  ginv <- function(x) ((x - L - W) / gm) + H 
  m    <- function(x) ((W / (H - L)) * (x - L)) + f(L)
  minv <- function(x) (((H - L) / W) * (x - L)) + f(L)

  if (inv) {
    function(x) ifelse(x < -g(H), ginv(x) - 2*ginv(0),
                       ifelse((-g(H) <= x) & (x < -L), minv(x) - 2*minv(0),
                              ifelse((-L <= x) & (x < L), f(x),
                                     ifelse((L <= x) & (x < g(H)), minv(x),
                                            ifelse(g(H) <= x, ginv(x), 0)))))
  } else {
    function(x) ifelse(x < -H, g(x) - 2*g(0),
                       ifelse((-H <= x) & (x < -L), m(x) - 2*m(0),
                              ifelse((-L <= x) & (x < L), f(x),
                                     ifelse((L <= x) & (x < H), m(x),
                                            ifelse(H <= x, g(x), x)))))
  }
}

rule_distribution <- function() {
  data <- read.csv("../completed/mat-mul_30x30_30x30/1/rule_distribution.csv")

  f <- function(avg, diff) {
    ifelse((diff < 25) & between(avg, 5, 1000),
           "pre-compile",
           ifelse((-0.1 < diff) & between(avg, 2000, 3000),
                  "compile",
                  ifelse(between(diff, -5, 5) & (avg < 9),
                         "opt",
                         "<none>")))
  }

  p <- data %>%
    pivot_wider(names_from=name, values_from=value) %>%
    ggplot(aes(
      x = average,
      y = differential,
      color = f(average, differential)
    )) +
    geom_jitter(
      position = position_jitter(
        seed=10,
        width=0.5,
        height=0.5
      ),
      size=4,
      alpha=0.5
    ) +
    scale_x_continuous(
      breaks=c(seq(0, 30, by=5), 2020, 2025),
      limits=c(0, 2025),
      trans=scales::trans_new(
        "custom",
        piecewise(30, 2020, 10, gm=1),
        piecewise(30, 2020, 10, gm=1, inv=T),
        )
    ) +
    scale_y_continuous(
      breaks=c(seq(-10, 10, by=5), seq(-4045, -4035, by=5), seq(4035, 4045, by=5)),
      limits=c(-4045, 4045),
      trans=scales::trans_new(
        "custom",
        piecewise(13, 4035, 10),
        piecewise(13, 4035, 10, inv=T),
        )
    ) +
    theme_classic() +
    theme(
      legend.position = "top",
      legend.background = element_blank(),
      legend.text = element_text(size=8, face="bold"),
      legend.title = element_blank(),
      legend.key.size = unit(0.75, "lines"),
      legend.box.spacing = unit(0, "lines"), 
      legend.margin = margin(2, 0, 2, 0),

      panel.grid.major.x = element_line(color="grey", linewidth=0.4),
      panel.grid.minor.x = element_line(color="grey", linewidth=0.1),
      panel.grid.major.y = element_line(color="grey", linewidth=0.4),
      panel.grid.minor.y = element_line(color="grey", linewidth=0.1),

      panel.background = element_blank(),
      panel.spacing.x = unit(0.4, "lines"),

      strip.placement = "outside",
      strip.background = element_blank(),
      strip.text = element_text(size=8, face="bold", margin=margin(0, 0, 4, 0)),

      plot.margin = margin(0, 0, 3, 0)
    ) +
    labs(x="Average", y="Differential", color="Phase")

  pb <- ggplot_build(p)
  pg <- ggplot_gtable(pb)

  data2npc <- function(x, axis, panel = 1L) {
    range <- pb$layout$panel_params[[panel]][[paste0(axis,".range")]]
    scales::rescale(c(range, x), c(0,1))[-c(1,2)]
  }
  
  axisBreakGrob <- function(coord, axis, size=0.02, sep=0.01, ...) {
    if (axis == "x") {
      grobTree(
        rectGrob(x=data2npc(coord, "x"), y=0, width=sep, height=sep, gp=gpar(lwd=0)),
        segmentsGrob(y0=-size, y1=size,
                     x0=data2npc(coord, "x"),
                     x1=data2npc(coord, "x")+sep,
                     gp=gpar(...)),
        segmentsGrob(y0=-size, y1=size,
                     x0=data2npc(coord, "x")-sep,
                     x1=data2npc(coord, "x"),
                     gp=gpar(...))
      )
    } else if (axis == "y") {
      grobTree(
        rectGrob(x=0, y=data2npc(coord, "y"), width=sep, height=sep, gp=gpar(lwd=0)),
        segmentsGrob(x0=-size, x1=size,
                     y0=data2npc(coord, "y"),
                     y1=data2npc(coord, "y")+sep,
                     gp=gpar(...)),
        segmentsGrob(x0=-size, x1=size,
                     y0=data2npc(coord, "y")-sep,
                     y1=data2npc(coord, "y"),
                     gp=gpar(...))
      )
    } else {
      stop("Invalid axis.")
    }
  }

  pg <- gtable_add_grob(
    pg,
    grobTree(
      axisBreakGrob(17, "y"),
      axisBreakGrob(-17, "y"),
      axisBreakGrob(35, "x")
    ),
    t=9, l=5,
    clip="off"
  )

  grid.newpage()
  grid.draw(pg)
}
