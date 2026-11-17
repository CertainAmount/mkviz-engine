# [Nasdaq Correction: 31 Since 1971, and How They Ended](https://mkviz.com/nasdaq-correction-history/)

## Research note

**Question.** When the Nasdaq Composite closes 10% or more below its record, how often does the fall stop short of a bear market (20%), how long do the two kinds take to recover, and does getting back above the 10% line mean the fall is over?

**Short answer.** Of the 31 completed Nasdaq drops of 10% or more since 1971, 18 stopped before 20% and 13 became bear markets. Of the 26 that closed back above the 10% line before reaching 20%, eight still became bear markets. The current drop: 11.9% below the Oct. 6 record at the Nov. 5 close, back above the line on Nov. 9, 11.2% below on Nov. 16.

### Data
- Nasdaq Composite daily closes from February 1971: FRED (NASDAQCOM) through Nov. 13, 2026, and Yahoo Finance (^IXIC) through Nov. 16, 2026; the two agree on every close checked in November 2026.
- NBER recession months (USREC) via FRED.

### Method
A drop starts on the first close at least 10% below the prior record close and ends at the next record. Depth is the lowest close in between. "Back above the line" is the first later close within 10% of the record, before any new record. Medians are across completed drops; the current drop is not counted.

```
./engine/target/release/mkviz-engine drawdowns data/series/index_ixic_long.csv --threshold 0.10 --deep 0.20 --flag data/series/fred_USREC.csv
```
(The `back`, `low_at_back`, `back_before_deep` and `back_then_deep` fields were added to `drawdowns` for this update.)

### Results

| | Count |
|---|---|
| Completed drops of 10% or more since 1971 | 31 |
| Stopped before 20% | 18 |
| Became bear markets | 13 |
| Bear markets without a US recession | 7 |
| Got back above the 10% line before 20% | 26 |
| ... and still became bear markets | 8 |

- Corrections that stopped short bottomed a median of 5.5 days after the first 10% close and got back to a record a median of 116 days after the peak. Bear markets took a median of 524 days.
- The first close back above the line came a median of two days after the first 10% close.
- The eight that got back above the line and still fell 20% or more started in 1983, 1989, 1998, 2000, 2018, 2020, 2021 (bottom 2022) and 2024 (bottom 2025).

### Limits
- Price index, no dividends; closes only, not intraday.
- The 10% and 20% lines are conventions, not turning points the market knows about.
- 31 drops is a small sample, and the 1970s Nasdaq was a much smaller, different index.

### Correction
- 2026-11-17: the Nov. 5 fall was first given as 4.9% and the close as 11.4% below the record, from a figure before the official close. The final close (24,315.00) makes it 5.5% and 11.9%.

### Sources
- FRED, NASDAQ Composite Index (NASDAQCOM): https://fred.stlouisfed.org/series/NASDAQCOM
- Yahoo Finance, Nasdaq Composite (^IXIC) history: https://finance.yahoo.com/quote/%5EIXIC/history/
- FRED, USREC: https://fred.stlouisfed.org/series/USREC

### Builds on and leads to
- [Stock market corrections since 1950](https://mkviz.com/stock-market-corrections-history/): the same count for the S&P 500, from 5% dips.

### Cite
MKViz (2026). *Nasdaq correction history.* https://mkviz.com/nasdaq-correction-history/ ; code and data: https://github.com/CertainAmount/mkviz-engine/tree/main/data/nasdaq-correction-history

## Charts

Run from the repository root after `cd engine && cargo build --release && cd ..`.

## nasdaq-drops-dots

Data: [nasdaq-drops-dots.csv](nasdaq-drops-dots.csv) · settings: [nasdaq-drops-dots.json](nasdaq-drops-dots.json)

![nasdaq-drops-dots](nasdaq-drops-dots.svg)

Drawn from market prices we don't redistribute; the chart data is in the CSV.
