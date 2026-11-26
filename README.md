# mkviz-engine

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.24025690.svg)](https://doi.org/10.5281/zenodo.24025690)

The calculation engine and chart data behind [MKViz](https://mkviz.com), a blog that explains money news with long-run official data.

Every number in an MKViz article that comes from our own calculation is produced by this engine, and every chart's data is published here so anyone can check it.

![Is Inflation Going Down? Only 2 of 6 Past Rebounds Faded](data/is-inflation-going-down/inflation-rebound-outcomes.svg)

**Latest articles**

- [Is Inflation Going Down? Only 2 of 6 Past Rebounds Faded](https://mkviz.com/is-inflation-going-down/) · [data](data/is-inflation-going-down/)
- [Gold Price History Since 1971: Is Gold Expensive Now?](https://mkviz.com/gold-price-history/) · [data](data/gold-price-history/)
- [Bonds Go Up When Stocks Go Down 15 Times in 18. Not in 2022](https://mkviz.com/do-bonds-go-up-when-stocks-go-down/) · [data](data/do-bonds-go-up-when-stocks-go-down/)
- [Why Bond Funds Lose Money in a Year Like 2026](https://mkviz.com/how-bond-funds-lose-money/) · [data](data/how-bond-funds-lose-money/)
- [Stock Market Recovery Usually Takes 2 Years. After Inflation, Up to 13](https://mkviz.com/stock-market-recovery-time/) · [data](data/stock-market-recovery-time/)

## What's here

- `engine/` — a small Rust program that reads official time series (FRED, BLS, EIA, World Bank, Census) and answers "how did this go in the past?": drawdowns and recoveries, where today ranks in history, bull and bear phases, event studies, seasonality, policy-rate cycles, stretches above a line, mortgage payments and more. It also draws the charts as SVG.
- `data/<article>/` — for each article, the data behind each chart (`.csv`), the exact engine settings that drew it (`.json`), the chart itself (`.svg`) and a README with the command that redraws it.
- `data/series/` — the official input series those charts were drawn from, as downloaded (public-domain US government data; World Bank data under CC BY 4.0).
- `METHODS.md` — what each calculation does and why.

## Run it

```
cd engine
cargo build --release
./target/release/mkviz-engine drawdowns ../data/series/fred_NASDAQCOM.csv --threshold 0.10
./target/release/mkviz-engine position ../data/series/fred_MSPUS.csv --afford ../data/series/fred_MORTGAGE30US.csv,../data/series/fred_MEHOINUSA646N.csv
```

Each result is JSON with a one-line `method` and a `provenance` block (engine version, input file, rows, date range, options). To redraw a chart, pass the `params` from its `.json` file back to `figure <frame>`.

Some charts use market prices from non-official sources; for those we publish the chart data but not the raw price series.

## License

Code: MIT. Chart data: free to reuse; please credit "MKViz" and link to the article.
