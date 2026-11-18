//! need: 대출 금리가 목표(예: 3%)까지 오려면 10년 국채가 어디까지 내려와야 하나, 그런 때가 얼마나 있었나,
//! 그리고 지금 같은 금리에서 시작한 주들이 N년 안에 그 목표까지 내려간 적이 있나.
//! 입력은 같은 주의 대출 금리(rate)와 그 주 국채(base, 조사 기간 평균)를 짝지은 값.

use crate::stats;

pub struct Week {
    pub date: String,
    pub day: i64,
    pub rate: f64,
    pub base: f64,
}

/// 정렬된 값의 p 분위(0~1, 가장 가까운 순위).
pub fn quantile(v: &[f64], p: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() - 1) as f64 * p).round() as usize]
}

pub struct Target {
    pub target: f64,
    pub base_needed: f64,      // 보통(중앙값) 금리 차일 때 필요한 국채
    pub base_needed_tight: f64, // 좁은 금리 차(하위 10%)일 때
    pub base_needed_wide: f64,  // 넓은 금리 차(상위 10%)일 때
    pub base_share: f64,        // 국채가 base_needed 이하였던 주의 몫
    pub base_last: Option<String>,
    pub rate_share: f64, // 대출 금리가 목표 이하였던 주의 몫
    pub rate_weeks: usize,
    pub rate_first: Option<String>,
    pub rate_last: Option<String>,
    pub starts_reached: usize, // 지금과 비슷한 금리에서 시작해 years년 안에 목표 이하로 간 시작 주 수
}

pub struct Need {
    pub gap_now: f64,
    pub gap_med: f64,
    pub gap_q10: f64,
    pub gap_q90: f64,
    pub targets: Vec<Target>,
    pub starts: usize,           // 비교한 시작 주 수(지금 금리 ± near, 뒤로 years년이 다 있는 주)
    pub start_years: Vec<i32>,   // 그 시작 주들의 해(중복 없이)
    pub lowest_after: Option<f64>, // 그 시작 주들의 years년 안 최저 금리 중 가장 낮은 것
    pub lowest_after_from: Option<String>,
    pub median_low: Option<f64>, // years년 안 최저 금리의 중앙값
}

/// 지금 금리 ± near에서 시작해 years년 뒤가 자료 안에 다 있는 주마다 (시작 날짜, 그 기간 최저 금리)
pub fn similar_lows(w: &[Week], years: f64, near: f64) -> Vec<(String, f64)> {
    let now = w.last().unwrap();
    let horizon = (years * 365.25) as i64;
    w.iter().enumerate()
        .filter(|(_, x)| (x.rate - now.rate).abs() <= near && x.day + horizon <= now.day)
        .map(|(i, x)| (x.date.clone(), w[i..].iter().take_while(|y| y.day <= x.day + horizon).map(|y| y.rate).fold(f64::MAX, f64::min)))
        .collect()
}

pub fn need(w: &[Week], targets: &[f64], years: f64, near: f64) -> Need {
    let gaps: Vec<f64> = w.iter().map(|x| x.rate - x.base).collect();
    let now = w.last().unwrap();
    let (gap_med, gap_q10, gap_q90) = (stats::median(&gaps).unwrap(), quantile(&gaps, 0.1), quantile(&gaps, 0.9));
    let horizon = (years * 365.25) as i64;
    let last_day = now.day;
    // 시작 주: 금리가 지금 ± near, 그 뒤 years년이 자료 안에 다 있는 주. 그 기간 최저 금리.
    let lows: Vec<(usize, f64)> = w.iter().enumerate()
        .filter(|(_, x)| (x.rate - now.rate).abs() <= near && x.day + horizon <= last_day)
        .map(|(i, x)| (i, w[i..].iter().take_while(|y| y.day <= x.day + horizon).map(|y| y.rate).fold(f64::MAX, f64::min)))
        .collect();
    let mut start_years: Vec<i32> = lows.iter().filter_map(|(i, _)| w[*i].date.get(..4)?.parse().ok()).collect();
    start_years.dedup();
    let lowest = lows.iter().fold(None::<(usize, f64)>, |a, &(i, l)| match a { Some((_, b)) if b <= l => a, _ => Some((i, l)) });
    let tg = targets.iter().map(|&t| {
        let need = t - gap_med;
        let hits: Vec<&Week> = w.iter().filter(|x| x.rate <= t).collect();
        Target {
            target: t,
            base_needed: need,
            base_needed_tight: t - gap_q10,
            base_needed_wide: t - gap_q90,
            base_share: stats::rank(&w.iter().map(|x| x.base).collect::<Vec<_>>(), need),
            base_last: w.iter().rev().find(|x| x.base <= need).map(|x| x.date.clone()),
            rate_share: hits.len() as f64 / w.len() as f64,
            rate_weeks: hits.len(),
            rate_first: hits.first().map(|x| x.date.clone()),
            rate_last: hits.last().map(|x| x.date.clone()),
            starts_reached: lows.iter().filter(|(_, l)| *l <= t).count(),
        }
    }).collect();
    Need {
        gap_now: now.rate - now.base,
        gap_med, gap_q10, gap_q90,
        targets: tg,
        starts: lows.len(),
        start_years,
        lowest_after: lowest.map(|(_, l)| l),
        lowest_after_from: lowest.map(|(i, _)| w[i].date.clone()),
        median_low: stats::median(&lows.iter().map(|(_, l)| *l).collect::<Vec<_>>()),
    }
}

#[cfg(test)]
mod need_test {
    use super::*;

    fn wk(d: i64, rate: f64, base: f64) -> Week {
        Week { date: format!("{}-01-01", 2000 + d / 365), day: d, rate, base }
    }

    #[test]
    fn needed_base_and_reach() {
        // 금리 차는 늘 2. 첫해 7%에서 시작해 둘째 해 3%까지 내려갔다가 다시 7%.
        let w: Vec<Week> = (0..30).map(|k| {
            let d = k * 73; // 5주에 한 번꼴, 30개 = 약 6년
            let r = if (5..10).contains(&k) { 3.0 } else { 7.0 };
            wk(d, r, r - 2.0)
        }).collect();
        let n = need(&w, &[3.0, 5.0], 2.0, 0.25);
        assert!((n.gap_med - 2.0).abs() < 1e-9);
        assert!((n.targets[0].base_needed - 1.0).abs() < 1e-9); // 3% - 2 = 국채 1%
        assert_eq!(n.targets[0].rate_weeks, 5);
        // 7%에서 시작해 2년 안에 3%까지 간 시작 주는 처음 다섯(0~4번 주)뿐이다
        assert_eq!(n.targets[0].starts_reached, 5);
        assert_eq!(n.lowest_after, Some(3.0));
    }
}
