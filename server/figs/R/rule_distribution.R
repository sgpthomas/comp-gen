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

rule_distribution <- function(alpha_v=12, beta_v=10) {
  data <- read.csv("../completed/mat-mul_30x30_30x30/1/rule_distribution.csv")

  ## function the determines which phase a rule falls into
  f <- function(avg, diff) {
    ifelse((diff < alpha_v) & (beta_v <= avg),
           "Expansion",
           ifelse(alpha_v <= diff,
                  "Compilation",
                  ifelse((diff < alpha_v) & (avg < beta_v),
                         "Optimization",
                         "<none>")))
  }

  xscale     <- piecewise(60, 4040, 10)
  xscale_inv <- piecewise(60, 4040, 10, inv=T)

  yscale     <- piecewise(13, 4035, 5)
  yscale_inv <- piecewise(13, 4035, 5, inv=T)

  p <- data %>%
    pivot_wider(names_from=name, values_from=value) %>%
    filter(differential > -4000) %>%
    ggplot(aes(
      x = average * 2,
      y = differential,
      fill = factor(f(average * 2, differential), levels=c("Expansion", "Compilation", "Optimization")),
      shape = factor(f(average * 2, differential), levels=c("Expansion", "Compilation", "Optimization"))
    )) +
    geom_jitter(
      position = position_jitter(
        seed=10,
        width=1,
        height=1
      ),
      size=3,
      alpha=0.6,
      stroke=0.1
    ) +
    geom_hline(
      yintercept=min(alpha_v, 4041),
      linetype="dashed",
      linewidth=1,
      color="salmon"
    ) +
    annotate(
      "text",
      x=ifelse(between(beta_v, 30, 50), 10, 45),
      y=min(alpha_v, 4033)+ifelse(between(alpha_v, 10, 4020), 2000, 5),
      label=str_c("$\\alpha=$", alpha_v),
      size=3
    ) +
    geom_segment(
      x=xscale(beta_v), y=yscale(alpha_v),
      xend=xscale(beta_v),
      yend=-Inf,
      linetype="dashed",
      linewidth=1,
      color="salmon"
    ) +
    annotate("text", x=xscale(beta_v+6), y=yscale(-7.5), label=str_c("$\\beta=$", beta_v), size=3) +
    scale_x_continuous(
      breaks=c(seq(0, 60, by=10), 4040, 4050),
      limits=c(0, 4050),
      trans=scales::trans_new("custom", xscale, xscale_inv)
    ) +
    scale_y_continuous(
      breaks=c(seq(-10, 10, by=5), seq(-4045, -4035, by=5), seq(4035, 4050, by=5)),
      limits=c(-15, 4041),
      trans=scales::trans_new("custom", yscale, yscale_inv)
    ) +
    labs(x="Aggregate Cost", y="Cost Differential", color="Phase", shape="Phase", fill="Phase") +
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
  scale_fill_brewer(
    palette = "Set2"
  ) +
  scale_shape_manual(
    values = c(21, 23, 22)
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
