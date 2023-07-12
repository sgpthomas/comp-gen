asplos24spring Paper #251 Reviews and Comments
===========================================================================
Paper #251 Automatic Generation of Vectorizing Compilers for Customizable
Digital Signal Processors


Review #251A
===========================================================================

Overall merit
-------------
4. Weak accept

Reviewer expertise
------------------
2. Some familiarity

Paper summary
-------------
This paper introduces a framework that automates the generation of a vectorizing compiler for Digital Signal Processors (DSPs). The framework contributes by automatically generating rewrite rules for vectorized instructions, categorizing them into phases based on a cost function, and applying these rules to transform a scalar program into a vectorized program. The paper also addresses the issue of compilation tractability by applying rules with a timeout in a loop and subsequently improving them greedily.

Strengths
---------
I enjoyed reading the paper. It is well-written with examples and graphs. It solves an interesting problem by generating the compiler for the new DSPs architectures.

Detailed feedback
-----------------
I have some minor comments and questions regarding the evaluation section and the compiler algorithm presented in the paper.

First, the evaluation of the paper includes a relatively limited number of kernels. Also, there is no explicit explanation provided for the chosen range of input sizes. It would be interesting to see a more diverse set of applications or kernels to ensure the robustness of the proposed framework.

Secondly, I am puzzled by the underperformance of Matrix Mul in Figure 3 with the proposed approach compared to the hand-written Nature Kernels. It would be helpful to have a deeper analysis of the results to understand the reasons behind this discrepancy and if the proposed tool can handle such scenarios in future work.
 
The proposed framework excels primarily in handling small and irregular kernels. However, it remains unclear how well this framework can adapt to scenarios involving both long-running and small kernels.

Regarding the algorithm presented in Figure 2, the compiler first applies expansion rules and then compilation rules. I am curious to know if expansion rules modify a program in a way that it is no more vectorizable. It would be valuable to gain insight into any such scenarios encountered during experimentation.



Review #251B
===========================================================================

Overall merit
-------------
5. Accept

Reviewer expertise
------------------
4. Expert

Paper summary
-------------
This paper presents an automated compiler synthesis framework for DSPs, that is powered by equality saturation (EqSat). The proposed tool, Isaria, uses an offline phase to synthesize rewrite rules for DSPs, which are then fed to an EqSat driven compiler. The rewrites themselves are applied in three separate phases to ensure scalability to real world programs.

The evaluation shows that Isaria is able to generate optimized DSPs that are, in several cases, comparable with the state-of-the-art tool, Diospyros, even though the Isaria-based vectorizing compiler is entirely automatically synthesized, including the rewrite rules and the schedule in which they are applied.

Strengths
---------
- Automated rule scheduling is a very interesting problem (even in EqSat) and this paper addresses this in the context of DSPs
- Evaluation is good (comparable results to hand-written rewrite rules, which is not easy!)
- Paper solves an important problem and makes several interesting observations

Weaknesses
----------
- Unclear what DSL is being used and how big of a grammar is used to synthesize rewrite rules.
- Paper does not clearly describe how/if Isaria was able to reduce the number of rules synthesized and by how much
- The cost-based rule scheduling part is not clearly explained, even though it is a core contribution

Detailed feedback
-----------------
I enjoyed reading this work. Even though EqSat has the reputation of mitigating phase ordering, in real applications, rule scheduling ends up being really important for scalability. I was excited to see this paper address this in the DSP context. Isaria automatically synthesizes rewrite rules for vectorizing DSP compilers together with a schedule for applying EqSat in phases. The automatic schedule synthesis part of Isaria is interesting and addresses tractability problems commonly encountered in EqSat driven tools. The idea of phased EqSat is general and definitely has applications in other EqSat tools. 

As the authors pointed out, tools like Ruler, while being good at generating small, effective rulesets, can still produce too many rules. The trick of using only 1-wide instructions during rule synthesis (but still verifying them using the ISA spec) together with rule scheduling is pretty neat and works well in practice.

DSPs seem to be a perfect domain for applying phased EqSat --- the authors identified three distinct phases of rule application based on insights about how rule application changes the original input program. While this sacrifices completeness, it is more practical and makes EqSat tractable. Another observation made by the authors (which is common in other EqSat applications too) is that a good ruleset must contain rules that both grow *and* shrink the egraph --- while shrinking the egraph is intuitive (it controls the search space), growing the egraph is also important as exploration only happens during growing the egraph. The observation about "short-circuiting" rules being generated by automated tools in Section 5.2 is interesting and also observed in other tools that use EqSat with automatically generated rulesets.

Using cost as a heuristic for phase assignment is a reasonable choice. However, it wasn't clear exactly how Isaria inspects subexpressions to assign costs. How is this different from a simple monotonic cost function? Overall, this part of the paper was a bit underwhelming, especially given that the key idea of the paper is applying EqSat in phases guided by an abstract cost model. For example,  how are $\alpha$ and $\beta$ being chosen? Some form of ablation of those values should be done in order to justify their values (which are never really mentioned anywhere in the paper as far as I can tell).  Is it possible that some rules are good for all three phases? What do you do if that happens?
 
On a related note, the paper should clearly specify what DSL it is operating on and some description of the semantics. Merely mentioned Diospyros is not helpful as it prevents the paper from being self-contained.  This also raise more questions for example how big of a grammar were you enumerating over, when using Ruler?  

The paper made this point that using Ruler directly was producing 300 rules rules whereas Diospyros had 28 hand-written rules. Having so many rules is hard for scaling EqSat applications. However, the paper does not say much about how Isaria is reducing the number of rules generated and to what? Is the main trick just using 1-lane instructions? How effective is it? How many rules did you end up with ultimately? How much do you think rule scheduling helped compared to reducing the number of synthesized rules, when it comes to scaling EqSat?

Why is the compile time for Isaria lower for QrD in Figure 4? For the other examples, how big is the egraph that it causes such a big slowdown?

What are some limitations of Isaria and how do you plan to address them?

A relevant paper that should be cited and discussed is Sketch-Guided EqSat(https://arxiv.org/abs/2111.13040).

## Minor comment
- Section 5: "Isaria-based compilers" -> I assume you mean a single compiler, right?



Review #251C
===========================================================================

Overall merit
-------------
5. Accept

Reviewer expertise
------------------
4. Expert

Paper summary
-------------
This paper describes Isaria, a new system that automatically creates vectorizing compilers given an architectural description and cost function.  Isaria automatically generates valid rewrite rules based on these semantics (via a lightly modified Ruler) and then uses these generated rules to automatically create a compiler.  The key insight is that to avoid the blowup due to large numbers of generated rules, Isaria automatically divides rules into phases, deconstructing the compilation into a series of applications of equality saturation, while greedily pruning to avoid exponential blowup.

Strengths
---------
- Very well written and understandable, a joy to read.
- Simple insight plus building on existing tools allows the system to tractably solve a difficult problem
- Good ablations

Weaknesses
----------
- Results are less impressive due to clang's much improved autovectorization, but this is a minor issue.
- Cannot handle horizontal operations

Detailed feedback
-----------------
The key insight in this paper is that it is possible to divide rewrite rules into phases, based on simple properties of the rules, enabling pruning and allowing equality saturation to scale to hundreds of automatically generated rules.  In addition, by building on Ruler and Diospyros, the authors reuse multiple pieces and can compare against prior work easily.

Overall, I enjoyed reading this paper.  The paper is well-explained and the evaluation does a good job of showcasing the system's strengths and weaknesses.

While no fault of the author's, the fact that autovectorization in clang has improved for regular kernels presents a question: can the two strategies be combined, e.g. by having one of the rewrite rules essentially call out to clang to (attempt to) autovectorize?  It seems the main hurdle may be that Isaria requires unrolled loops and so cannot compactly represent some of the vectorizations used by clang.

Additionally, it would be good if the authors could characterize how future work could mitigate the biggest weakness of the system w.r.t. the kinds of vector instructions the system can support.  Although Tensilica DSPs seem to have few non-element-wise instructions, other DSPs such as Hexagon HVX have a rich set of operations that combine element-wise operations with shuffles or do horizontal reductions.  Would an approach such as taken by Ahmad et al [1] where just a _limited_ number of lanes are reasoned about work?  How much impact would this have on scaling?



Review #251D
===========================================================================

Overall merit
-------------
5. Accept

Reviewer expertise
------------------
3. Knowledgeable

Paper summary
-------------
While equality saturation and related synthesis techniques have found their way to vectorizing compilers, the state of the art still involves a wealth of human intervention. It is particularly limiting in the context of DSP architectures, with ISAs implementing a lot of domain-specific vector instructions, and varying quickly (or even customer-dependent). The proposed approach and tool, called Isaria, extends previous work on rule synthesis for vectorization and on equality-saturation-based vectorization, tackling the scalability and human-in-the-loop limitations of the state of the art.

Strengths
---------
Leveraging compiler know-how and coarse-grained phase decomposition in an equality saturation algorithm is very novel and potentially disruptive.

Strong and elegant theoretical model.

Weaknesses
----------
The performance claims in the introduction need to be tone down. It is impossible to generalize the results on kernels to larger programs without a more extensive study, including a scalability study (e.g., playing with upstream unroll factors).

Limited performance evaluation: the selected kernels match those of the most closely related equality saturation approach, but this is not really justifyable in a context where further automation has been achieved, enabling (in theory) a broader range of behaviors. I don't understand the rationale for such a limited range of kernels.

Detailed feedback
-----------------
Questions.

What is the cause of the 30% performance drop compared to manually
crafter rewrite rules in Diospyros?

Scalability beyond small kernels? E.g., would it be possible to evaluate the effect of loop unrolling (upstream, prior to the application of equality saturation)?

Why not compare with SLP vectorization on common benchmark suites? The version of clang used as a baseline is not up to speed with more advanced academic prototypes.

Why considering only DSPs and not conventional CPU architectures?  I understand the motivation is stronger on DSPs. But a broader evaluation would make the comparison much stronger and teachful (especially on the limits and remaining challenges).

Details.

one day seconds -> one day
