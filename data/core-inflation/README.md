# [Core Inflation vs Headline: Which Predicts Prices Better?](https://mkviz.com/core-inflation/)

## Research note

**Question.** When gas prices push headline inflation above core inflation (the consumer price index without food and energy), which one is the better guide to where inflation will be a year later? And what does it mean when core itself starts to speed up, as it did in fall 2026?

**Short answer.** Since 1985, in months when headline ran at least a point above core, core was the closer guess for headline a year later 64% of the time. That edge disappears when the spike spreads beyond energy (late 2021) and, counting 1958-1984, overall. When core's three-month pace ran a point or more above its 12-month rate, core's 12-month rate was higher a year later in 6 of 10 years; the rises were sometimes large, the declines small. In October 2026 core's three-month pace was 3.75% against an estimated yearly rate of 2.4% to 2.7%.

### Data
- CPI-U all items (CPIAUCSL) and all items less food and energy (CPILFESL), seasonally adjusted, BLS via FRED, monthly since 1947 and 1957.
- October 2026 12-month rates are MKViz estimates (see the [October inflation note](https://github.com/CertainAmount/mkviz-engine/tree/main/data/october-inflation-rate)).

### Test 1: which guess was closer a year later
For every month since January 1985 with headline 12-month inflation at least one point above core, we compared each with headline inflation 12 months later.

| | Value |
|---|---|
| Months tested | 77 |
| Core closer | 64% |
| Median miss, core as the guess | 1.4 points |
| Median miss, headline as the guess | 1.8 points |
| Core closer, all months since 1985 | 53% |
| Core closer, split months including 1958-1984 | 52% |

Big splits and what followed:

| Month | Headline | Core | Headline a year later |
|---|---|---|---|
| Sept. 2005 | +4.7% | +2.0% | +2.1% |
| July 2008 | +5.6% | +2.5% | -2.1% |
| Sept. 2011 | +3.9% | +2.0% | +2.0% |
| Dec. 2021 | +7.0% | +5.5% | +6.5% |
| June 2022 | +9.1% | +5.9% | +3.0% |

```
./engine/target/release/mkviz-engine closer data/series/fred_CPIAUCSL.csv --a data/series/fred_CPILFESL.csv --b data/series/fred_CPIAUCSL.csv --change 12 --ahead 12 --from 1985-01-01 --min-gap 0.01
```

### Test 2: what followed when core sped up
Pace = core's change over the last three months, annualized. For every month since 1985 when the pace ran at least one point above core's 12-month rate (22 months, 16 of them followed by a higher 12-month rate a year later), we took the first such month in each year:

| Core first ran a point ahead | Core, 12 months | A year later |
|---|---|---|
| April 1987 | 3.90% | 4.26% |
| March 1990 | 4.79% | 5.24% |
| March 1995 | 2.96% | 2.81% |
| March 2004 | 1.56% | 2.35% |
| May 2006 | 2.44% | 2.27% |
| Aug. 2011 | 1.97% | 1.94% |
| Feb. 2018 | 1.88% | 2.14% |
| Aug. 2020 | 1.72% | 3.94% |
| April 2021 | 2.98% | 6.16% |
| Jan. 2022 | 6.05% | 5.54% |

```
./engine/target/release/mkviz-engine accel data/series/fred_CPILFESL.csv --short 3 --gap 0.01 --ahead 12 --from 1985-01
```

### Context in fall 2026
- Core rose 0.3% in October after seasonal adjustment, as in August and September (BLS). Three-month pace: 1.64% in July, 3.75% in October.
- Airline fares rose 22% in the year to September (seasonally adjusted); transportation services rose 0.8% in October.

### Limits
- The tests use the CPI; the Federal Reserve's 2% goal uses the PCE price index.
- Episodes overlap (late 2021 and early 2022 count as two years of one run).
- Twelve months ahead is one horizon; results at other horizons may differ.

### Sources
- BLS, Consumer Price Index Summary, October 2026: https://www.bls.gov/news.release/cpi.nr0.htm
- FRED, CPIAUCSL and CPILFESL: https://fred.stlouisfed.org/series/CPILFESL
- Federal Reserve, Why does the Federal Reserve aim for inflation of 2 percent: https://www.federalreserve.gov/faqs/economy_14400.htm

### Builds on and leads to
- Uses the October 2026 estimates from the [October inflation note](https://github.com/CertainAmount/mkviz-engine/tree/main/data/october-inflation-rate).
- Open: repeat Test 1 for the PCE index; add services-less-rent ("supercore") as a third guess; update Test 2 monthly.

### Cite
MKViz (2026). *Core inflation vs headline: which predicts prices better.* https://mkviz.com/core-inflation/ ; code and data: https://github.com/CertainAmount/mkviz-engine/tree/main/data/core-inflation

## Charts

Run from the repository root after `cd engine && cargo build --release && cd ..`.

## core-vs-headline-miss

Data: [core-vs-headline-miss.csv](core-vs-headline-miss.csv) · settings: [core-vs-headline-miss.json](core-vs-headline-miss.json)

![core-vs-headline-miss](core-vs-headline-miss.svg)

```
./engine/target/release/mkviz-engine figure closer data/series/fred_CPIAUCSL.csv --a data/series/fred_CPILFESL.csv --ahead 12 --axis 'How far the guess missed inflation a year later (points)' --b data/series/fred_CPIAUCSL.csv --change 12 --from 1985-01-01 --min-gap 0.01 --names 'Guess with core,Guess with headline' --title 'Core was usually the closer guess a year later' --to 2025-09-01 --out core-vs-headline-miss.svg
```
