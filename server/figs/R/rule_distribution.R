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

rule_distribution <- function(alpha=12, beta=10) {
  data <- read.csv("../completed/mat-mul_30x30_30x30/1/rule_distribution.csv")

  f <- function(avg, diff) {
    ifelse(alpha <= diff,
           "Compilation",
           ifelse(beta <= avg,
                  "Expansion",
                  ifelse(avg < beta,
                                "Optimization",
                                "<none>")))
  }

  p <- data %>%
    pivot_wider(names_from=name, values_from=value) %>%
    filter(differential > -4000) %>%
    ggplot(aes(
      x = average * 2,
      y = differential,
      color = factor(f(average * 2, differential), levels=c("Expansion", "Compilation", "Optimization"))
    )) +
    geom_jitter(
      position = position_jitter(
        seed=10,
        width=1,
        height=1
      ),
      size=3,
      alpha=0.6,
    ) +
    geom_hline(
      yintercept=min(alpha, 4041),
      linetype="dashed",
      linewidth=1,
      color="salmon"
    ) +
    annotate(
      "text",
      x=ifelse(between(beta, 30, 50), 10, 45),
      y=min(alpha, 4033)+ifelse(between(alpha, 10, 4020), 2000, 5),
      label=str_c("$\\alpha=$", alpha),
      size=3
    ) +
    geom_vline(
      xintercept=beta,
      linetype="dashed",
      linewidth=1,
      color="salmon"
    ) +
    annotate("text", x=beta+6, y=4037.5, label=str_c("$\\beta=$", beta), size=3) +
    scale_x_continuous(
      breaks=c(seq(0, 60, by=10), 4040, 4050),
      limits=c(0, 4050),
      trans=scales::trans_new(
        "custom",
        piecewise(60, 4040, 10, gm=1),
        piecewise(60, 4040, 10, gm=1, inv=T),
        )
    ) +
    scale_y_continuous(
      breaks=c(seq(-10, 10, by=5), seq(-4045, -4035, by=5), seq(4035, 4050, by=5)),
      limits=c(-15, 4041),
      trans=scales::trans_new(
        "custom",
        piecewise(13, 4035, 5),
        piecewise(13, 4035, 5, inv=T),
        )
    ) +
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
      legend.margin = margin(2, 0, 2, 0),

      panel.grid.major.x = element_line(color="grey", linewidth=0.4),
      panel.grid.major.y = element_line(color="grey", linewidth=0.4),

      panel.background = element_blank(),
      panel.spacing.x = unit(0.4, "lines"),

      strip.placement = "outside",
      strip.background = element_blank(),
      strip.text = element_text(size=8, face="bold", margin=margin(0, 0, 4, 0)),

      plot.margin = margin(0, 0, 3, 0)
    ) +
  labs(x="Aggregate Cost", y="Cost Differential", color="Phase") +
  scale_color_brewer(
    palette = "Set2"
  )

  pb <- ggplot_build(p)
  pg <- ggplot_gtable(pb)

  data2npc <- function(x, axis, panel = 1L) {
    range <- pb$layout$panel_params[[panel]][[paste0(axis,".range")]]
    scales::rescale(c(range, x), c(0,1))[-c(1,2)]
  }
  
  axisBreakGrob <- function(xcoord, ycoord, len=0.02, sep=0.01, theta=45, ...) {
    cx <- ifelse(is.na(xcoord), 0, data2npc(xcoord, "x"))
    cy <- ifelse(is.na(ycoord), 0, data2npc(ycoord, "y"))

    a <- len * cos(theta * (pi/180))
    b <- len * sin(theta * (pi/180))

    xsep <- ifelse(is.na(xcoord), 0, sep)
    ysep <- ifelse(is.na(ycoord), 0, sep)

    grobTree(
      rectGrob(x=cx, y=cy, width=1.5*sep, height=1.5*sep, gp=gpar(lwd=0, fill="white")),
      segmentsGrob(x0=(cx - xsep) - a, y0=(cy - ysep)-b,
                   x1=(cx - xsep) + a, y1=(cy - ysep)+b,
                   gp=gpar(...)),
      segmentsGrob(x0=(cx + xsep) - a, y0=(cy + ysep)-b,
                   x1=(cx + xsep) + a, y1=(cy + ysep)+b,
                   gp=gpar(...))
    )
  }

  pg <- gtable_add_grob(
    pg,
    grobTree(
      axisBreakGrob(NA, 14, lwd=2, theta=60, sep=0.013),
      axisBreakGrob(65, NA, lwd=2, theta=70, sep=0.006)
      ## axisBreakGrob(14, "y", len=0.02, lwd=2),
      ## axisBreakGrob(32.5, "x", len=0.02, sep=0.006, theta=75, lwd=2)
    ),
    t=9, l=5,
    clip="off"
  )

  grid.newpage()
  grid.draw(pg)
}
