# Methods

All inputs are files with a date column and a value column. Dates are compared as calendar days; monthly comparisons match the same calendar month (a missing month never shifts the comparison).

## Calculations

| Command | What it does |
|---|---|
| `drawdowns` | Every fall of at least the threshold from a prior record, followed to the lowest close and to a new record. Splits episodes into those that stopped short of the deeper line and those that went past it. |
| `position` | Where the latest value ranks among all observations, the record low and high and their dates, the median, and the last time the series was this low or high. `--change N` ranks the N-month change instead. |
| `turning` | Bull and bear phases dated with the Pagan and Sossounov (2003) rules on month-end values. |
| `sahm` | The Sahm rule: three-month average unemployment rate minus its low over the prior 12 months. |
| `events` | Change over set windows after each event date, compared with the same-length change from every start date (the base rate). |
| `spikes` | First day a series closed a threshold above its recent low, then what followed. |
| `recession-prob` | Probit model of a recession a set number of months ahead on the 10-year minus 3-month Treasury spread, fitted on our data (in-sample). |
| `analogs` | The past months closest to today on several standardized indicators. |
| `passthrough` | After each big move in a leading series (oil), how much of it a following series (gasoline) passed through, rises and falls separately. |
| `cycles` | Policy-rate cycles: consecutive target-rate changes in one direction. |
| `seasonal` | Each month's average as a percent above or below its year's average; median by month and share of years above. |
| `during` | For every big fall in another series, the change in this one over the same dates. |
| `spells` | Every stretch at or above (or below) a line: start, end, length, most extreme value, and whether it is still under way. |
| `payment` | Fixed-rate monthly payment: loan × r / (1 − (1 + r)^−n), r = annual rate / 12. |

## Transforms (any command)

| Option | What it does |
|---|---|
| `--splice <old>` | Chain-links an older series in front of a newer one, matched on the average of the linking month. |
| `--deflate <cpi>` | Restates values in today's dollars. |
| `--avg N` | Trailing N-point average. |
| `--per <series>` | Divides by another series (latest value on or before each date). |
| `--afford <rate>,<income>` | Turns a home-price series into the monthly principal and interest (20% down, 30 years by default; `--down`, `--years`) as a share of monthly income. Taxes and insurance are not included. |

## Charts (`figure <frame>`)

`underwater`, `paths`, `band`, `dots`, `fan`, `response`, `cases`, `during`, `pictogram`, `strip`, `multiples`, `gauge`, `calendar`, `timeline`, `spells`. Every chart also writes its data (`.csv`) and settings (`.json`) next to the SVG.

Charts follow two findings from perception research: compare values by position on a shared axis (Cleveland and McGill 1984), and highlight only one thing, usually today (Borkin et al. 2013).
