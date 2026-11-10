# [Why Are Mortgage Rates So High? The Gap Over Treasuries Is Back](https://mkviz.com/mortgage-rates-treasury-gap/)

Run from the repository root after `cd engine && cargo build --release && cd ..`.

## mortgage-rate-stack

Data: [mortgage-rate-stack.csv](mortgage-rate-stack.csv) · settings: [mortgage-rate-stack.json](mortgage-rate-stack.json)

![mortgage-rate-stack](mortgage-rate-stack.svg)

```
./engine/target/release/mkviz-engine figure stack data/series/fred_MORTGAGE30US.csv --base data/series/fred_DGS10.csv --dates 2021-01-07,2023-10-26,2025-11-06,2026-11-05 --labels 'Jan 2021,Oct 2023,Nov 2025,This week' --names '10-year Treasury,Gap on top' --title 'What a 30-year mortgage rate is made of: the 10-year Treasury yield plus the gap lenders and investors add' --typical 1.67 --out mortgage-rate-stack.svg
```
