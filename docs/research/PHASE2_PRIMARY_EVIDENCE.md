# Phase 2 primary research evidence

- **Evidence date:** 2026-10-25
- **Scope:** Primary sources behind the Phase 2 feature/baseline hypotheses and validation controls
- **Governing registry:** [Phase checklist, Section 5.2](../PHASE_CHECKLIST.md#52-research-paper-registry)
- **Status:** Bibliographic identity and reported scope verified; no result has been reproduced on NSE CM data

This is a supporting evidence note, not a second roadmap or registry. It records what the cited sources actually establish, what they do not establish, and the bounded experiment each source motivates for JFT.

## Evidence rule

- **CONFIRMED FROM PRIMARY SOURCE** means the bibliographic record, dataset, method, or reported result is present in the publisher record, author manuscript, or primary preprint linked below.
- **RESEARCH HYPOTHESIS** means JFT may test the relationship on qualified NSE CM data; the external result is not presumed transferable.
- **UNRESOLVED / REQUIRES VALIDATION** means representative current NSE data, a predeclared protocol, or an execution-grade simulator is still required.
- Citation verification does not authorize Phase 2 before Phase 1 acceptance and does not establish economic value after costs.

<a id="rp-001"></a>

## RP-001 - Order-flow imbalance and short-horizon price changes

**Citation and publication status — CONFIRMED FROM PRIMARY SOURCE**

Rama Cont, Arseniy Kukanov, and Sasha Stoikov, “The Price Impact of Order Book Events,” *Journal of Financial Econometrics* 12(1), 47-88 (2014), DOI [`10.1093/jjfinec/nbt003`](https://doi.org/10.1093/jjfinec/nbt003). The publisher records publication on 7 June 2013 for the Winter 2014 issue. The authors' primary preprint is [`arXiv:1011.6402`](https://arxiv.org/abs/1011.6402).

**Dataset/period and method — CONFIRMED FROM PRIMARY SOURCE**

- Consolidated NYSE TAQ trades and quotes for 50 randomly selected S&P 500 constituents.
- Twenty-one trading days in April 2010.
- The paper defines order-flow imbalance (OFI) from best-bid/best-ask order-book events, relates short-interval price changes to OFI, and examines the interaction with market depth.

**Reported result — CONFIRMED FROM PRIMARY SOURCE**

The paper reports an approximately linear short-interval relationship between price changes and OFI, with the fitted slope inversely related to depth. It reports stability across the sampled stocks, time scales, and intraday seasonality adjustments; its volume-only relationship is described as noisier and less robust.

**Limitations**

- One month of U.S. consolidated quote/trade data is not evidence of persistence on NSE CM.
- Best-quote OFI is a descriptive/predictive relationship, not a fill model, causal claim, or net trading strategy.
- Aggregation interval, event ordering, venue fragmentation, tick size, latency, and cost assumptions can materially change the result.

**JFT reproduction and outcome**

- **RESEARCH HYPOTHESIS:** Signed OFI computed only from accepted canonical book transitions predicts a future mid-price or executable-price response beyond a naive no-change baseline.
- Predeclare event-count and wall-clock horizons, compute features strictly as-of the decision point, fit only on training windows, and report coefficient sign, monotonicity, calibration/error, depth interaction, and stability by instrument/session/regime.
- Keep prediction separate from trading policy and evaluate costs/fills only in Phase 3.
- **JFT outcome:** Accepted as a Phase 2 interpretable feature and linear-baseline candidate, not as evidence that an NSE strategy works.

<a id="rp-002"></a>

## RP-002 - Queue imbalance and the next mid-price move

**Citation and publication status — CONFIRMED FROM PRIMARY SOURCE**

Martin D. Gould and Julius Bonart, “Queue Imbalance as a One-Tick-Ahead Price Predictor in a Limit Order Book,” *Market Microstructure and Liquidity* 2(2), 1650006 (2016), DOI [`10.1142/S2382626616500064`](https://doi.org/10.1142/S2382626616500064). The primary manuscript is [`arXiv:1512.03492`](https://arxiv.org/abs/1512.03492). The DOI ends in `0064`.

**Dataset/period and method — CONFIRMED FROM PRIMARY SOURCE**

- Event-by-event LOBSTER data for ten liquid Nasdaq stocks over all 252 trading days in 2014.
- The study uses 10:00-15:30, excluding the first and last 30 minutes; five stocks are treated as large-tick and five as small-tick.
- Logistic and local-logistic models use best bid/ask queue imbalance to predict the direction of the next mid-price change.

**Reported result — CONFIRMED FROM PRIMARY SOURCE**

The paper reports out-of-sample ROC AUCs around 0.7-0.8 for the sampled large-tick names and about 0.6-0.65 for the sampled small-tick names. It also reports larger reductions in probabilistic squared error for large-tick names than small-tick names. Local logistic fits add little over the simpler logistic model.

**Limitations**

- The label is the next mid-price move, not a fixed-time return or executable outcome.
- The sample excludes auctions and market-open/close periods and contains U.S. price-time-priority venue data.
- Statistical classification improvement does not include spread, fees, latency, fill probability, or adverse selection.

**JFT reproduction and outcome**

- **RESEARCH HYPOTHESIS:** Top-of-book queue imbalance improves calibrated next-move probabilities over class-prior and no-change baselines on qualified NSE CM data.
- Use chronological instrument/day splits; report log loss, Brier score, calibration, class-conditional recall, AUC only as a secondary diagnostic, and results by relative tick size and session.
- Compare ordinary logistic regression before any nonlinear model. Do not copy the paper's thresholds or claim tradability from classification alone.
- **JFT outcome:** Accepted as the first interpretable Phase 2 classification baseline.

<a id="rp-003"></a>

## RP-003 - Volume imbalance as a signal and policy input

**Citation and publication status — CONFIRMED FROM PRIMARY SOURCE**

Álvaro Cartea, Ryan Donnelly, and Sebastian Jaimungal, “Enhancing Trading Strategies with Order Book Signals,” *Applied Mathematical Finance* 25(1), 1-35 (2018), DOI [`10.1080/1350486X.2018.1434009`](https://doi.org/10.1080/1350486X.2018.1434009). The [Oxford University Research Archive](https://ora.ox.ac.uk/objects/uuid%3A006addde-3a03-4d75-89c1-04b59026e1c0) contains the accepted author manuscript and confirms the journal metadata.

**Dataset/period and method — CONFIRMED FROM PRIMARY SOURCE**

- High-frequency Nasdaq data for eleven equities.
- January-June 2014 is used in sample; July-December 2014 is used out of sample.
- The paper studies volume imbalance as a predictor of the next market-order sign and of price changes immediately after a market order. It then embeds the signal in a calibrated Markov-chain-modulated jump model and a limit-order control problem with inventory penalties.

**Reported result — CONFIRMED FROM PRIMARY SOURCE**

The authors report that adding their imbalance state improves the tested strategy's profits by reducing adverse-selection exposure and positioning orders for favorable moves.

**Limitations**

- The economic result depends on the paper's model, calibration, queue/execution assumptions, objective, and Nasdaq sample.
- A model-based policy result cannot establish attainable NSE queue priority, latency, fill probability, or net returns.
- The paper combines a predictive finding with a control policy; copying both at once would make failure attribution difficult.

**JFT reproduction and outcome**

- **RESEARCH HYPOTHESIS:** Imbalance predicts order-flow direction and immediate price response on NSE CM after controlling for tick size, spread, depth, session, and time of day.
- Test the predictive component in Phase 2. Treat any quoting/execution policy as a separate Phase 3 experiment under explicit fill and cost cases.
- **JFT outcome:** Supports the existing separation between predictor, execution policy, and hard risk; it does not authorize adoption of the paper's strategy.

<a id="rp-004"></a>

## RP-004 - DeepLOB as a later representation-learning benchmark

**Citation and publication status — CONFIRMED FROM PRIMARY SOURCE**

Zihao Zhang, Stefan Zohren, and Stephen Roberts, “DeepLOB: Deep Convolutional Neural Networks for Limit Order Books,” *IEEE Transactions on Signal Processing* 67(11), 3001-3012 (2019), DOI [`10.1109/TSP.2019.2907260`](https://doi.org/10.1109/TSP.2019.2907260). The primary preprint is [`arXiv:1808.03668`](https://arxiv.org/abs/1808.03668), and the [Oxford University Research Archive](https://ora.ox.ac.uk/objects/uuid%3A4411af59-2657-4e3e-8ee2-81032c37671c) confirms the final journal metadata.

**Dataset/period and method — CONFIRMED FROM PRIMARY SOURCE**

- FI-2010: five Nasdaq Nordic stocks over ten consecutive days.
- LSE: top ten book levels from 3 January through 24 December 2017 for five training names, with a three-month test period and additional transfer tests on five unseen names.
- The architecture combines convolutional layers, an inception module, and an LSTM over the most recent 100 book states to classify smoothed future mid-price direction.

**Reported result — CONFIRMED FROM PRIMARY SOURCE**

The paper reports stronger classification results than its listed FI-2010 baselines and stable LSE out-of-sample classification, including transfer to instruments excluded from training. It also explicitly notes that FI-2010 alone is too short and downsampled for adequate modern validation.

**Limitations**

- Label definitions smooth future prices and materially determine the task; they must not be imported without leakage analysis.
- Classification accuracy on FI-2010/LSE is not net economic uplift on NSE CM.
- The paper's simple trading assessment does not replace JFT's execution-grade simulator, conservative costs, capacity tests, or simpler-model comparison.

**JFT reproduction and outcome**

- **RESEARCH HYPOTHESIS:** A compact spatial/temporal encoder may add stable net out-of-sample value beyond accepted linear/tree/small-MLP baselines.
- Reproduction is deferred to Phase 4, after a Phase 3 net-positive simple baseline. Use JFT labels/splits, multiple seeds, calibration, inference latency, ablation, and net uplift under the identical policy.
- **JFT outcome:** Accepted only as a later benchmark family. It is explicitly not a Phase 2 implementation target.

<a id="rp-005"></a>

## RP-005 - Leakage, data snooping, and repeated-search controls

**Primary sources — CONFIRMED FROM PRIMARY SOURCE**

1. Halbert White, “A Reality Check for Data Snooping,” *Econometrica* 68(5), 1097-1126 (2000), DOI [`10.1111/1468-0262.00152`](https://doi.org/10.1111/1468-0262.00152). The paper tests whether the best model found in a specification search is superior to a benchmark while accounting for reuse of the same history.
2. Peter R. Hansen, “A Test for Superior Predictive Ability,” *Journal of Business & Economic Statistics* 23(4), 365-380 (2005), DOI [`10.1198/073500105000000063`](https://doi.org/10.1198/073500105000000063). The SPA test modifies the Reality Check with studentization and a sample-dependent null to reduce sensitivity to poor alternatives.
3. David H. Bailey, Jonathan M. Borwein, Marcos López de Prado, and Qiji Jim Zhu, “The Probability of Backtest Overfitting,” *Journal of Computational Finance* 20(4), 39-69 (2017), DOI [`10.21314/JCF.2016.322`](https://doi.org/10.21314/JCF.2016.322). The paper proposes combinatorially symmetric cross-validation and a probability-of-backtest-overfitting diagnostic across tried configurations.
4. Marcos López de Prado, *Advances in Financial Machine Learning* (Wiley, 2018), ISBN `978-1-119-48210-9`. Chapters 7 and 12 are a practitioner source for purging, embargo, walk-forward limitations, and combinatorial purged cross-validation. This is a book/method reference, not an independent empirical proof that a chosen split is valid.

**Evidence basis and method — CONFIRMED FROM PRIMARY SOURCE**

- These are methodological references rather than one transferable market-effect study, so there is no common security sample or market period to reproduce.
- White specifies a benchmark-relative bootstrap test for the best model found during specification search.
- Hansen evaluates the studentized SPA test with Monte Carlo experiments and a U.S.-inflation forecasting exercise.
- Bailey et al. evaluate combinatorially symmetric cross-validation on investment-simulation examples and estimate how often in-sample selection underperforms out of sample.
- López de Prado documents implementation patterns for purging observations with overlapping information, embargoing adjacent observations, walk-forward validation, and combinatorial purged cross-validation.

**Limitations and JFT application**

- These methods address different failure modes; none repairs look-ahead features, incorrect event ordering, unrealistic fills, unlogged experimentation, or repeatedly opened holdouts.
- Purge and embargo lengths must be derived from JFT feature lookbacks, label horizons, and overlapping outcomes; the sources do not supply NSE-specific values.
- White/SPA-style tests require a defined benchmark, loss differential, dependence-aware bootstrap choices, and the complete candidate family. PBO requires the full configuration-performance matrix; reporting it only for selected winners is invalid.
- **UNRESOLVED / REQUIRES VALIDATION:** Exact split geometry, bootstrap method/block length, family definition, minimum fold count, acceptance thresholds, and the locked final-OOS policy must be predeclared after label horizons and sample size are known.
- **JFT outcome:** The sources justify mandatory chronological splits, leakage tests, a complete trial ledger, benchmark-relative multiple-testing diagnostics, and a holdout that becomes consumed once inspected. They do not justify a universal numeric threshold today.

## Evidence-backed Phase 2 entry gate

This citation review closes bibliographic verification, not the Phase 1 data gate. Before Phase 2 implementation:

1. Phase 1 must accept representative current NSE CM replay and canonical schemas.
2. Each feature and label must identify its information time, lookback, horizon, overlap, eligibility, and failure behavior.
3. The experiment protocol must predeclare chronological train/validation/test windows, purge/embargo derived from those contracts, baselines, primary metrics, subgroup slices, and stopping rules.
4. Every attempted feature/model/configuration must enter the trial ledger, including failed and abandoned trials.
5. Predictive evidence must be reported separately from execution and net-P&L evidence.

## Deliberately still open

RP-006 (market impact, queue position, fill probability, and latency) remains open because it governs Phase 3 simulator assumptions rather than Phase 2 feature plausibility. It requires a separate primary-source review plus broker/venue/current-cost evidence before simulator parameters can be frozen.
