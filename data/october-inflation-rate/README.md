# [October Inflation Rate: The Number BLS Couldn't Publish](https://mkviz.com/october-inflation-rate/)

Run from the repository root after `cd engine && cargo build --release && cd ..`.

## october-inflation-shift

Data: [october-inflation-shift.csv](october-inflation-shift.csv) · settings: [october-inflation-shift.json](october-inflation-shift.json)

![october-inflation-shift](october-inflation-shift.svg)

```
./engine/target/release/mkviz-engine figure shift data/series/bls_CUUR0000SA0.csv --from 1985-01-01 --missing 2025-10 --names 'Midpoint of Sept. and Nov. 2025 (BLS),Treasury'\''s stand-in for TIPS' --ref-label 'September rate' --title 'October'\''s annual inflation rate, two ways' --out october-inflation-shift.svg
```
