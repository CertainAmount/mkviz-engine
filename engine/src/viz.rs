//! 비주얼 엔진: 분석 결과를 '프레임'(정해진 꼴의 그림 틀)에 채워 SVG로 낸다(Grammar of Graphics: 데이터 → 위치·색 매핑 → 표시).
//! 지각 연구(Cleveland & McGill 1984): 비교는 같은 축 위의 위치로. 기억(Borkin 외 2013): 강조는 하나(지금 사례)만.
//! 프레임은 이 파일에 함수 하나씩 더한다. 폭 600 viewBox, 글자 12 이상, 사이트 팔레트(METHODS.md B4·B5).

use crate::series::{Point, day_number};
use std::fmt::Write;

const W: f64 = 480.0; // 지침 §4-2: 휴대폰 폭에서 글자가 줄지 않게(600이면 7px까지 줄었다, 2026-11-06)
const H: f64 = 280.0;
const L: f64 = 44.0;
const R: f64 = 16.0;
const T: f64 = 14.0;
const B: f64 = 26.0;
const BLUE: &str = "#2563eb";
const GRAY: &str = "#94a3b8";
const SHADE: &str = "#e2e8f0";
const INK: &str = "#475569";

fn head(title: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 {W} {H}\" width=\"{W}\" height=\"{H}\" style=\"width:100%;max-width:560px;height:auto\" role=\"img\" xmlns=\"http://www.w3.org/2000/svg\" font-family=\"system-ui,sans-serif\" font-size=\"16\"><title>{}</title>",
        title.replace('&', "&amp;").replace('<', "&lt;")
    )
}

fn pct(v: f64) -> String {
    let r = (v * 100.0).round();
    format!("{}%", if r == 0.0 { 0.0 } else { r }) // -0% 대신 0%
}

/// 고점 대비 하락폭(물속 그림): 0선 아래로 얼마나 잠겼는지. 기준선(예: -10%, -20%)과 flag(경기침체) 음영.
pub fn underwater(pts: &[Point], lines: &[f64], flag: &[Point], marks_in: &[(i64, f64)], title: &str) -> String {
    let mut peak = f64::MIN;
    let dd: Vec<(i64, f64)> = pts.iter().map(|p| { peak = peak.max(p.value); (p.day, p.value / peak - 1.0) }).collect();
    let (x0, x1) = (dd[0].0 as f64, dd.last().unwrap().0 as f64);
    let lo = dd.iter().map(|d| d.1).fold(0.0, f64::min).min(-0.1);
    let lo = (lo * 10.0).floor() / 10.0;
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R);
    let y = |v: f64| T + (0.0 - v) / (0.0 - lo) * (H - T - B);
    let mut s = head(title);
    // 경기침체 음영(월간 flag가 1인 달)
    for f in flag.iter().filter(|f| f.value >= 1.0 && f.day as f64 >= x0 && f.day as f64 <= x1) {
        let _ = write!(s, "<rect x=\"{:.1}\" y=\"{T}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{SHADE}\"/>", x(f.day as f64), (x(f.day as f64 + 31.0) - x(f.day as f64)).max(1.0), H - T - B);
    }
    let mut v = 0.0;
    while v >= lo - 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#e5e7eb\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - R, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v -= 0.2;
    }
    for &t in lines {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(-t), y(-t));
        if ((t * 100.0).round() as i64) % 20 != 0 {
            let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", L - 6.0, y(-t) + 4.0, pct(-t));
        }
    }
    let mut path = format!("M{:.1},{:.1}", x(x0), y(0.0));
    for (d, v) in &dd {
        let _ = write!(path, "L{:.1},{:.1}", x(*d as f64), y(*v));
    }
    let _ = write!(path, "L{:.1},{:.1}Z", x(x1), y(0.0));
    let _ = write!(s, "<path d=\"{path}\" fill=\"{BLUE}\" fill-opacity=\"0.25\" stroke=\"{BLUE}\" stroke-width=\"1.2\"/>");
    // 기준선(가장 얕은 lines)보다 깊었던 하락마다 바닥 아래에 그 해를 적는다(그림만 떼어 봐도 어느 하락인지 읽히게)
    let cut = lines.iter().cloned().fold(f64::MAX, f64::min);
    if cut < f64::MAX {
        let mut seg: Option<(i64, f64)> = None;
        let mut marks: Vec<(i64, f64)> = vec![];
        for &(d, v) in dd.iter().chain(std::iter::once(&(i64::MAX, 0.0))).filter(|_| marks_in.is_empty()) {
            if v < 0.0 {
                if seg.map_or(true, |m| v < m.1) { seg = Some((d, v)); }
            } else if let Some(m) = seg.take() {
                if m.1 <= -cut { marks.push(m); }
            }
        }
        if !marks_in.is_empty() {
            // 원래 해상도(일별)의 하락 기록에서 받은 바닥: 그림은 월평균이라도 숫자는 본문 표와 같게
            marks = marks_in.iter().filter(|m| m.1 <= -cut).cloned().collect();
        }
        // 이름표가 겹치면(가로 44 이내) 뒤의 것을 한 줄 아래로 내린다(지금 진행 중인 하락도 빠지지 않게)
        let mut placed: Vec<(f64, f64)> = vec![];
        for (d, v) in marks {
            let mut ly = (y(v) + 18.0).min(H - B - 4.0);
            while placed.iter().any(|&(px, py)| (x(d as f64) - px).abs() < 70.0 && (ly - py).abs() < 16.0) { ly += 17.0; }
            placed.push((x(d as f64), ly));
            let year = pts.iter().find(|p| p.day >= d).map(|p| &p.date[..4]).unwrap_or("");
            // 해와 깊이를 함께(그림만 봐도 '얼마나'가 읽히게, 주석 연구: Borkin 외 2016)
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{year} {}</text>", x(d as f64).clamp(L + 30.0, W - R - 30.0), ly, pct(v));
        }
    }
    // 지금 위치: 마지막 점에 점과 'Now'(독자가 '지금은 어디냐'를 바로 찾게)
    if let Some(&(d, v)) = dd.last() {
        if v < 0.0 {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{BLUE}\" stroke=\"#fff\" stroke-width=\"2\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-weight=\"600\" font-size=\"14\">Now {}</text>", x(d as f64), y(v), x(d as f64) - 8.0, y(v) - 10.0, pct(v));
        }
    }
    let (ya, yb) = (&pts[0].date[..4], &pts.last().unwrap().date[..4]);
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{ya}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{yb}</text></svg>", H - 6.0, W - R, H - 6.0);
    s
}

/// 사건 뒤 경로 겹치기: 각 사건일을 0으로 맞춘 뒤 days일까지의 변화. 지난 사례는 회색, 마지막(지금) 사례만 파랑.
/// diff: 비율 대신 수준 차이(금리 %p). labels: 선마다 끝에 붙일 이름(시작 연도 등, 비우면 없음).
pub fn paths_with(pts: &[Point], starts: &[String], days: i64, title: &str, diff: bool, labels: &[String]) -> String {
    let lines: Vec<Vec<(f64, f64)>> = starts
        .iter()
        .filter_map(|d| {
            let s = day_number(d)?;
            let i = pts.iter().position(|p| p.day >= s)?;
            let w: Vec<&Point> = pts[i..].iter().take_while(|p| p.day - pts[i].day <= days).collect();
            // 선 하나에 구간 60개 안팎(파일을 작게). 구간마다 최고·최저를 둘 다 남겨 꼭대기·바닥이 깎이지 않게(2000년 3월 고점이 사라졌다)
            let step = (w.len() / 60).max(1);
            let v = |p: &Point| if diff { p.value - pts[i].value } else { p.value / pts[i].value - 1.0 };
            let mut out: Vec<(f64, f64)> = vec![(0.0, 0.0)]; // 출발점은 늘 0
            for chunk in w.chunks(step) {
                let hi = chunk.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
                let lo = chunk.iter().min_by(|a, b| a.value.partial_cmp(&b.value).unwrap()).unwrap();
                let mut two = [hi, lo];
                two.sort_by_key(|p| p.day);
                for p in two { if out.last().map_or(true, |l| l.0 != (p.day - pts[i].day) as f64) { out.push(((p.day - pts[i].day) as f64, v(p))); } }
            }
            if let Some(last) = w.last() { if out.last().map_or(true, |l| l.0 != (last.day - pts[i].day) as f64) { out.push(((last.day - pts[i].day) as f64, v(last))); } }
            Some(out)
        })
        .collect();
    let all: Vec<f64> = lines.iter().flatten().map(|p| p.1).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(0.0, f64::min), all.iter().cloned().fold(0.0, f64::max));
    let unit = if diff { 2.0 } else { 10.0 }; // diff: 0.5%p 눈금
    lo = (lo * unit).floor() / unit;
    hi = (hi * unit).ceil() / unit;
    let right = if labels.is_empty() { R } else { R + 76.0 }; // 끝 이름표 + 값 두 줄 자리
    let lm = if hi >= 9.995 || lo <= -0.995 { L + 18.0 } else if hi >= 0.995 { L + 10.0 } else { L }; // 1000%·100% 눈금 글자가 잘리지 않게
    let x = |d: f64| lm + d / days as f64 * (W - lm - right);
    let y = |v: f64| T + (hi - v) / (hi - lo).max(0.01) * (H - T - B);
    let mut s = head(title);
    // 눈금 6개 이하(휴대폰에서 숫자가 겹치지 않게)
    let step = [0.05, 0.1, 0.2, 0.25, 0.5, 1.0, 2.0, 5.0].into_iter().map(|st| if diff { st * 10.0 } else { st }).find(|st| (hi - lo) / st <= 5.0).unwrap_or(1.0);
    let fmt = |v: f64| if diff { if v.abs() < 1e-9 { "0".to_string() } else { format!("{:+.1}", v) } } else { pct(v) };
    let mut v = (lo / step).ceil() * step; // 눈금은 0을 지나게
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{lm}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - right, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, lm - 6.0, y(v) + 4.0, fmt(v));
        v += step;
    }
    for (k, ln) in lines.iter().enumerate() {
        let last = k + 1 == lines.len();
        let d: Vec<String> = ln.iter().map(|(a, b)| format!("{:.1},{:.1}", x(*a), y(*b))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"{}\"/>", d.join(" "), if last { BLUE } else { GRAY }, if last { 2.5 } else { 1.0 }, if last { 1.0 } else { 0.6 });
        if let (Some(l), Some(end)) = (labels.get(k), ln.last()) {
            // 끝 이름표에 값까지(그림만 봐도 '얼마나'가 읽히게)
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"15\" font-weight=\"600\">{}</text><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                           x(end.0) + 6.0, y(end.1) - 2.0, if last { BLUE } else { INK }, l, x(end.0) + 6.0, y(end.1) + 15.0, if end.1 > 0.0 { format!("+{}", fmt(end.1)) } else { fmt(end.1) });
        }
    }
    // 선이 하나면 꼭대기와 바닥에 값을 단다(그림만 봐도 '얼마나 올랐다 얼마나 빠졌나'가 읽히게)
    if lines.len() == 1 {
        let ln = &lines[0];
        // 바닥 = 전체 최저, 꼭대기 = 그 바닥 앞의 최고(오른 뒤 빠진 모양을 짚는다)
        let low = ln.iter().cloned().fold((0.0, f64::MAX), |a, p| if p.1 < a.1 { p } else { a });
        let top = ln.iter().filter(|p| p.0 < low.0).cloned().fold((0.0, f64::MIN), |a, p| if p.1 > a.1 { p } else { a });
        if top.1 > f64::MIN {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">Peak +{}</text>",
                           x(top.0), y(top.1), x(top.0) + 8.0, y(top.1) + 5.0, fmt(top.1));
        }
        if low.1 < f64::MAX {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{BLUE}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">Low {}</text>",
                           x(low.0), y(low.1), x(low.0), y(low.1) + 20.0, fmt(low.1));
        }
    }
    let _ = write!(s, "<text x=\"{lm}\" y=\"{}\" fill=\"{INK}\">0</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{}</text></svg>", H - 6.0, W - right, H - 6.0, if days % 365 == 0 || days > 730 { let y = (days as f64 / 365.25).round() as i64; format!("{} year{}", y, if y > 1 { "s" } else { "" }) } else { format!("{days} days") });
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::parse;

    #[test]
    fn frames_are_valid_small_svg() {
        let p = parse("d,v\n2000-01-01,100\n2000-06-01,70\n2001-01-01,110\n");
        let u = underwater(&p, &[0.1, 0.2], &parse("d,v\n2000-03-01,1\n"), &[], "t");
        assert!(u.starts_with("<svg") && u.ends_with("</svg>") && u.contains("-20%") && u.contains("<rect"));
        let q = paths_with(&p, &["2000-01-01".into()], 400, "t", false, &[]);
        assert!(q.contains("polyline") && q.len() < 20_000);
    }
}

/// 역사 속 위치 띠: 전체 기간 값의 10~90% 범위(띠)와 중앙값(점선), 시계열 선, 지금 점 강조.
pub fn band(pts: &[Point], title: &str, as_pct: bool, suffix: &str) -> String {
    let mut vals: Vec<f64> = pts.iter().map(|p| p.value).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f64| vals[((vals.len() - 1) as f64 * f) as usize];
    // 극단값 몇 개(2020년 청구 건수 600만)가 나머지를 납작하게 만들지 않게 0.5~99.5% 범위로 자르고, 넘는 값은 가장자리에 붙인다
    let (lo, hi) = (q(0.005).min(pts.last().unwrap().value), q(0.995).max(pts.last().unwrap().value));
    let pad = (hi - lo) * 0.05;
    let (lo, hi) = (lo - pad, hi + pad);
    let (x0, x1) = (pts[0].day as f64, pts.last().unwrap().day as f64);
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R);
    let y = |v: f64| T + (hi - v.clamp(lo, hi)) / (hi - lo).max(1e-9) * (H - T - B);
    let lab = |v: f64| if as_pct { pct(v) } else if v.abs() >= 1000.0 { format!("{:.0}k", v / 1000.0) } else { format!("{:.1}{suffix}", v) };
    let mut s = head(title);
    let _ = write!(s, "<rect x=\"{L}\" y=\"{:.1}\" width=\"{}\" height=\"{:.1}\" fill=\"{SHADE}\"/>", y(q(0.9)), W - L - R, y(q(0.1)) - y(q(0.9)));
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(q(0.5)), y(q(0.5)));
    let mut last_y = f64::MIN;
    for v in [q(0.9), q(0.5), q(0.1)] {
        if y(v) - last_y < 14.0 {
            continue; // 눈금 글자가 겹치면 뺀다
        }
        last_y = y(v);
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", L - 6.0, y(v) + 4.0, lab(v));
    }
    let step = (pts.len() / 400).max(1);
    let d: Vec<String> = pts.iter().step_by(step).chain(pts.last()).map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{GRAY}\" stroke-width=\"1.2\"/>", d.join(" "));
    let l = pts.last().unwrap();
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{BLUE}\"/>", x(l.day as f64), y(l.value));
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{}</text></svg>", H - 6.0, &pts[0].date[..4], W - R, H - 6.0, &l.date[..4]);
    s
}

/// 사례 점 그림: 사례마다 점 하나(가로 = 시작 연도, 세로 = 깊이). 기준선 하나, 지금 사례만 파랑(Cleveland & McGill: 같은 축 위 위치).
pub fn dots(items: &[(i64, f64, bool)], line: f64, first_year: &str, last_year: &str, title: &str) -> String {
    let lo = items.iter().map(|i| i.1).fold(0.0, f64::min);
    let lo = (lo * 10.0).floor() / 10.0;
    let (x0, x1) = (items.iter().map(|i| i.0).min().unwrap_or(0) as f64, items.iter().map(|i| i.0).max().unwrap_or(1) as f64);
    let x = |d: f64| L + 6.0 + (d - x0) / (x1 - x0).max(1.0) * (W - L - R - 12.0);
    let y = |v: f64| T + (0.0 - v) / (0.0 - lo).max(1e-9) * (H - T - B);
    let mut s = head(title);
    let mut v = 0.0;
    while v >= lo - 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#e5e7eb\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>", W - R, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v -= 0.2;
    }
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R, y(-line), y(-line));
    for &(d, v, now) in items {
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>", x(d as f64), y(v), if now { 6 } else { 4 }, if now { BLUE } else { GRAY }, if now { 1.0 } else { 0.8 });
    }
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">{first_year}</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{last_year}</text></svg>", H - 6.0, W - R, H - 6.0);
    s
}

/// 사례 줄 그림: 사례마다 한 줄(위에서 아래로), 같은 가로축(변화율) 위에 도중 최저점·첫 시점·둘째 시점을 점으로.
/// 최저점에서 끝 점까지 옅은 선이 '도중에 빠졌다가 어디서 끝났나'를 보여 준다. 세로 점선은 기저율(B5: 같은 축 위치, A5: 평소와 견준다).
/// rows: (이름, 도중 최저, 첫 시점 값, 둘째 시점 값). names: [최저, 첫 시점, 둘째 시점] 범례 이름.
pub fn cases(rows: &[(String, f64, f64, Option<f64>)], base: f64, names: [&str; 3], title: &str) -> String {
    const ORANGE: &str = "#eb6834";
    // 왼쪽 이름표 길이에 맞춘 여백("Facebook 2012"가 잘렸다)
    let left = (rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(4) as f64 * 8.6 + 12.0).max(56.0);
    let (rh, top) = (24.0, 62.0);
    let h = top + rh * rows.len() as f64 + 30.0;
    let vals: Vec<f64> = rows.iter().flat_map(|r| [r.1.min(0.0), r.2, r.3.unwrap_or(0.0)]).collect();
    let lo = (vals.iter().cloned().fold(0.0, f64::min) * 10.0).floor() / 10.0;
    // 튀는 값 하나가 나머지를 한쪽에 몰지 않게: 가장 큰 값이 둘째의 두 배를 넘으면 축을 둘째 근처에서 끊고 그 점은 끝에 실제 값과 함께 둔다
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let cap = match sorted.as_slice() { [a, b, ..] if *a > 0.5 && *a > 2.0 * b.max(0.05) => Some(b.max(0.05)), _ => None };
    let hi = (cap.unwrap_or(sorted.first().copied().unwrap_or(0.0).max(0.0)) * 10.0 + 1.0).floor() / 10.0;
    let x = |v: f64| left + (v.min(hi) - lo) / (hi - lo).max(0.01) * (W - left - R);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    // 범례: 표시는 색, 글자는 잉크색. 이름이 빈 항목은 뺀다, 간격은 이름 길이로
    let mut lx = left;
    for (c, n) in [ORANGE, BLUE, GRAY].iter().zip(names).filter(|(_, n)| !n.is_empty()) {
        let _ = write!(s, "<circle cx=\"{}\" cy=\"18\" r=\"6\" fill=\"{c}\"/><text x=\"{}\" y=\"24\" fill=\"{INK}\">{}</text>", lx + 6.0, lx + 16.0, n.replace('&', "&amp;"));
        lx += 16.0 + n.chars().count() as f64 * 8.6 + 20.0;
    }
    // 눈금은 6개 이하(휴대폰에서 숫자가 겹치지 않게)
    let step = [0.05, 0.1, 0.2, 0.25, 0.5, 1.0].into_iter().find(|st| (hi - lo) / st <= 5.0).unwrap_or(1.0);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{}\" y2=\"{}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"{INK}\">{}</text>",
                       x(v), x(v), top - 8.0, h - 26.0, if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, x(v), h - 8.0, pct(v));
        v += step;
    }
    if !base.is_nan() {
    let base_label = if names[1].chars().count() <= 12 { format!("typical {}", names[1]) } else { "median".to_string() };
    let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{}\" y2=\"{}\" stroke=\"{BLUE}\" stroke-dasharray=\"4 3\"/><text x=\"{:.1}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{}</text>", x(base), x(base), top - 8.0, h - 26.0, x(base) + 4.0, top - 12.0, base_label);
    }
    for (k, (name, w, a, b)) in rows.iter().enumerate() {
        let y = top + rh * k as f64 + rh / 2.0;
        let ends = [w.min(0.0), *a, b.unwrap_or(*a)];
        let (l, r) = (ends.iter().cloned().fold(f64::MAX, f64::min), ends.iter().cloned().fold(f64::MIN, f64::max));
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text><line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{y:.1}\" y2=\"{y:.1}\" stroke=\"#cbd5e1\" stroke-width=\"2\"/>",
                       left - 8.0, y + 5.0, name.replace('&', "&amp;"), x(l), x(r));
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{ORANGE}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(w.min(0.0)));
        if let Some(b) = b {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{GRAY}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*b));
        }
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"6\" fill=\"{BLUE}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*a));
        if *a > hi {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">+{} →</text>", x(*a) - 10.0, y + 5.0, pct(*a));
        }
    }
    s.push_str("</svg>");
    s
}

/// 그림문자(ISOTYPE) 그림: 묶음마다 단위 점 total개(10개씩 줄), 그중 filled개를 칠한다. "100달러 중 17달러가 남는다"처럼
/// 비율을 셀 수 있는 물건으로 보여 준다(Haroz·Kosara·Franconeri 2015: 그림문자는 정확도를 해치지 않고 기억·몰입을 높인다).
/// groups: (이름, 칠할 수, 전체 수, 큰 글씨 값). 두 묶음까지 나란히, 그 이상은 아래로.
pub fn pictogram(groups: &[(String, usize, usize, String)], title: &str) -> String {
    let (cell, cols) = (18.0, 10usize);
    let per_row = if groups.len() <= 2 { groups.len().max(1) } else { 2 };
    let gw = cell * cols as f64;
    let rows_per = groups.iter().map(|g| g.2.div_ceil(cols)).max().unwrap_or(1) as f64;
    let gh = 44.0 + rows_per * cell + 30.0;
    let h = 8.0 + gh * groups.len().div_ceil(per_row) as f64;
    let gap = (W - gw * per_row as f64) / (per_row as f64 + 1.0);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, filled, total, big)) in groups.iter().enumerate() {
        let x0 = gap + (k % per_row) as f64 * (gw + gap);
        let y0 = 8.0 + (k / per_row) as f64 * gh;
        let color = if k == 0 { BLUE } else { "#eb6834" };
        let _ = write!(s, "<text x=\"{x0:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"28\" font-weight=\"700\">{}</text>", y0 + 28.0, big.replace('&', "&amp;"));
        for i in 0..*total {
            let (cx, cy) = (x0 + (i % cols) as f64 * cell + cell / 2.0, y0 + 44.0 + (i / cols) as f64 * cell + cell / 2.0);
            let _ = write!(s, "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"6.5\" fill=\"{}\"/>", if i < *filled { color } else { SHADE });
        }
        let _ = write!(s, "<text x=\"{x0:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"15\">{}</text>", y0 + 44.0 + rows_per * cell + 22.0, name.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

/// 세로 시간선: 사건마다 해·점·굵은 한 줄·결과 한두 줄. 점 색은 결과의 성격(좋음 초록·중간 노랑·나쁨 주황)이고 글자가 그 뜻을 함께 말한다.
/// 사례 결말이 제각각인 이야기(거래소 사고 뒤 이용자 돈)를 표보다 순서대로 읽히게. items: (해, 사건, 결과, 성격 good|mixed|bad).
pub fn timeline(items: &[(String, String, String, String)], title: &str) -> String {
    fn wrap(t: &str, n: usize) -> Vec<String> {
        let mut out: Vec<String> = vec![String::new()];
        for w in t.split_whitespace() {
            if out.last().unwrap().len() + w.len() + 1 > n && !out.last().unwrap().is_empty() { out.push(String::new()); }
            let l = out.last_mut().unwrap();
            if !l.is_empty() { l.push(' '); }
            l.push_str(w);
        }
        out
    }
    // 왼쪽 이름표(해·날짜) 길이에 맞춰 여백을 넓힌다("Sept. 15", "Summer"가 잘렸다)
    let lw = items.iter().map(|i| i.0.chars().count()).max().unwrap_or(4) as f64 * 9.0 + 10.0;
    let (xl, xd, xt) = (lw.max(52.0), lw.max(52.0) + 22.0, lw.max(52.0) + 40.0);
    let narrow = ((xt - 92.0) / 8.0) as usize;
    let blocks: Vec<(Vec<String>, Vec<String>)> = items.iter().map(|i| (wrap(&i.1, 40 - narrow.min(12)), wrap(&i.2, 46 - narrow.min(12)))).collect();
    let hs: Vec<f64> = blocks.iter().map(|(a, b)| 12.0 + a.len() as f64 * 20.0 + b.len() as f64 * 19.0 + 14.0).collect();
    let h = 16.0 + hs.iter().sum::<f64>();
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let _ = write!(s, "<line x1=\"{xd}\" x2=\"{xd}\" y1=\"20\" y2=\"{:.1}\" stroke=\"#e2e8f0\" stroke-width=\"2\"/>", h - 14.0);
    let mut y = 16.0;
    for ((year, _, _, tone), ((a, b), bh)) in items.iter().zip(blocks.iter().zip(&hs)) {
        let c = match tone.as_str() { "good" => "#1baf7a", "bad" => "#eb6834", _ => "#eda100" };
        let _ = write!(s, "<text x=\"{xl}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-weight=\"600\">{}</text><circle cx=\"{xd}\" cy=\"{:.1}\" r=\"7\" fill=\"{c}\" stroke=\"#fff\" stroke-width=\"3\"/>",
                       y + 16.0, year.replace('&', "&amp;"), y + 11.0);
        let mut ty = y + 16.0;
        for l in a { let _ = write!(s, "<text x=\"{xt}\" y=\"{ty:.1}\" fill=\"#0f172a\" font-weight=\"600\">{}</text>", l.replace('&', "&amp;")); ty += 20.0; }
        for l in b { let _ = write!(s, "<text x=\"{xt}\" y=\"{ty:.1}\" fill=\"{INK}\" font-size=\"15\">{}</text>", l.replace('&', "&amp;")); ty += 19.0; }
        y += bh;
    }
    s.push_str("</svg>");
    s
}

/// 달력 열지도: 해(줄) × 달(칸), 칸 색은 그해 평균 대비(아래 파랑, 위 주황, 0 근처 회색: 발산 색은 두 색 + 회색 가운데).
/// 맨 아래 줄은 달별 중앙값과 숫자. 계절이 해마다 되풀이되는지 한눈에(달력 열지도: Van Wijk & Van Selow 1999).
pub fn calendar(years: &[(i32, [f64; 12])], typical: &[f64], scale: f64, title: &str) -> String {
    let (left, top, cw) = (62.0, 30.0, (W - 62.0 - R) / 12.0);
    let rh = 11.0;
    let h = top + rh * years.len() as f64 + 10.0 + 30.0 + 26.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let color = |v: f64| {
        let t = (v / scale).clamp(-1.0, 1.0);
        let (r0, g0, b0) = (241.0, 245.0, 249.0);
        let (r1, g1, b1) = if t < 0.0 { (37.0, 99.0, 235.0) } else { (235.0, 104.0, 52.0) };
        let a = t.abs();
        format!("rgb({:.0},{:.0},{:.0})", r0 + (r1 - r0) * a, g0 + (g1 - g0) * a, b0 + (b1 - b0) * a)
    };
    for (k, m) in ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"].iter().enumerate() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{m}</text>", left + cw * (k as f64 + 0.5), top - 10.0);
    }
    for (r, (y, dev)) in years.iter().enumerate() {
        let yy = top + rh * r as f64;
        if y % 5 == 0 || r == 0 || r + 1 == years.len() {
            let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\">{y}</text>", left - 6.0, yy + 9.5);
        }
        for (k, v) in dev.iter().enumerate() {
            let _ = write!(s, "<rect x=\"{:.1}\" y=\"{yy:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>", left + cw * k as f64 + 1.0, cw - 2.0, rh - 1.5, color(*v));
        }
    }
    let ty = top + rh * years.len() as f64 + 10.0;
    let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"12\" font-weight=\"600\">Typical</text>", left - 6.0, ty + 19.0);
    for (k, v) in typical.iter().enumerate() {
        let x = left + cw * k as f64 + 1.0;
        let _ = write!(s, "<rect x=\"{x:.1}\" y=\"{ty:.1}\" width=\"{:.1}\" height=\"28\" rx=\"3\" fill=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{}\" font-size=\"12\" font-weight=\"600\">{}</text>",
                       cw - 2.0, color(*v), x + (cw - 2.0) / 2.0, ty + 19.0, if (v / scale).abs() > 0.5 { "#fff" } else { "#0f172a" }, if *v > 0.0 { format!("+{}", pct(*v)) } else { pct(*v) });
    }
    let ly = ty + 28.0 + 20.0;
    let _ = write!(s, "<rect x=\"{left}\" y=\"{:.1}\" width=\"14\" height=\"10\" fill=\"{}\"/><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Cheaper than that year's average</text><rect x=\"{}\" y=\"{:.1}\" width=\"14\" height=\"10\" fill=\"{}\"/><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\">Pricier</text>",
                   ly - 9.0, color(-scale), left + 20.0, ly, left + 260.0, ly - 9.0, color(scale), left + 280.0, ly);
    s.push_str("</svg>");
    s
}

/// 게이지: 반원 위 바늘 하나로 "지금은 역사 범위의 어디"(백분위). 양끝에 최저·최고 값과 해, 가운데 큰 글씨로 지금 값과 백분위.
/// 위치 하나를 길게 설명하는 대신 한눈에(작은 다이얼은 정확한 비교보다 '대략 어디'를 말할 때만 쓴다).
pub fn gauge(rank: f64, now_label: &str, lo_label: &str, hi_label: &str, title: &str) -> String {
    let (cx, cy, r) = (W / 2.0, 180.0, 150.0);
    let h = 262.0;
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let pt = |f: f64, rr: f64| { let a = std::f64::consts::PI * (1.0 - f); (cx + rr * a.cos(), cy - rr * a.sin()) };
    // 세 구간(하위 3분의 1 파랑, 가운데 회색, 상위 3분의 1 주황)
    for (a, b, c) in [(0.0, 1.0 / 3.0, "#bfdbfe"), (1.0 / 3.0, 2.0 / 3.0, "#e2e8f0"), (2.0 / 3.0, 1.0, "#fed7aa")] {
        let (x0, y0) = pt(a, r);
        let (x1, y1) = pt(b, r);
        let _ = write!(s, "<path d=\"M{x0:.1},{y0:.1} A{r},{r} 0 0 1 {x1:.1},{y1:.1}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"26\"/>");
    }
    let (nx, ny) = pt(rank.clamp(0.0, 1.0), r - 6.0);
    let _ = write!(s, "<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{nx:.1}\" y2=\"{ny:.1}\" stroke=\"#0f172a\" stroke-width=\"3\" stroke-linecap=\"round\"/><circle cx=\"{cx}\" cy=\"{cy}\" r=\"7\" fill=\"#0f172a\"/>");
    let _ = write!(s, "<text x=\"{cx}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"#0f172a\" font-size=\"22\" font-weight=\"700\">{}</text>", cy + 64.0, now_label.replace('&', "&amp;"));
    let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                   cx - r, cy + 26.0, lo_label.replace('&', "&amp;"), cx + r, cy + 26.0, hi_label.replace('&', "&amp;"));
    s.push_str("</svg>");
    s
}

/// 작은 그림 여러 개(small multiples): 사례마다 작은 선 하나, 모두 같은 축(고점 = 0에서 days일). 칸마다 해·바닥 값·끝 값.
/// 사례끼리 모양을 나란히 견준다(Tufte 1983: 같은 틀 안에서 바뀌는 것만 보이게). 마지막 칸(지금)은 파랑.
pub fn multiples(cases: &[(String, Vec<(f64, f64)>)], days: f64, title: &str) -> String {
    let cols = 2usize;
    let (cw, chh, gap) = ((W - 16.0) / cols as f64, 120.0, 10.0);
    let rows = cases.len().div_ceil(cols);
    let h = rows as f64 * (chh + gap) + 8.0;
    let lo = cases.iter().flat_map(|c| c.1.iter().map(|p| p.1)).fold(0.0, f64::min).max(-1.0);
    let hi = cases.iter().flat_map(|c| c.1.iter().map(|p| p.1)).fold(0.0, f64::max).min(1.0); // 튀는 사례 하나(+639%)가 나머지를 납작하게 만들지 않게 위는 +100%에서 자른다(끝 값 글자는 실제 값)
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, pts)) in cases.iter().enumerate() {
        let (x0, y0) = (8.0 + (k % cols) as f64 * cw, 4.0 + (k / cols) as f64 * (chh + gap));
        let (px0, px1, py0, py1) = (x0 + 6.0, x0 + cw - 50.0, y0 + 26.0, y0 + chh - 10.0);
        let x = |d: f64| px0 + d / days * (px1 - px0);
        let y = |v: f64| py0 + (hi - v.clamp(lo, hi)) / (hi - lo).max(0.01) * (py1 - py0);
        let now = k + 1 == cases.len();
        let c = if now { BLUE } else { INK };
        let _ = write!(s, "<text x=\"{px0:.1}\" y=\"{:.1}\" fill=\"{c}\" font-size=\"15\" font-weight=\"600\">{}</text>", y0 + 16.0, name.replace('&', "&amp;"));
        let _ = write!(s, "<line x1=\"{px0:.1}\" x2=\"{px1:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#cbd5e1\"/>", y(0.0), y(0.0));
        let d: Vec<String> = pts.iter().map(|(a, b)| format!("{:.1},{:.1}", x(*a), y(*b))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>", d.join(" "), if now { BLUE } else { "#64748b" });
        if let Some(low) = pts.iter().cloned().reduce(|a, b| if b.1 < a.1 { b } else { a }) {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3\" fill=\"#eb6834\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\" text-anchor=\"middle\">{}</text>", x(low.0), y(low.1), x(low.0), (y(low.1) + 15.0).min(py1 + 8.0), pct(low.1));
        }
        if let Some(end) = pts.last() {
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{c}\" font-size=\"13\" font-weight=\"600\">{}{}</text>", px1 + 4.0, y(end.1) + 4.0, if end.1 > 0.0 { "+" } else { "" }, pct(end.1));
        }
    }
    s.push_str("</svg>");
    s
}

/// 띠 점 그림(strip plot): 모든 사례를 한 축 위의 점으로, 강조 묶음은 파랑(이름표), 나머지는 회색. 묶음마다 중앙값 세로선.
/// "이 묶음이 나머지보다 높은가"를 개별 점과 함께 보여 준다(평균 막대 두 개보다 분포가 보인다, Weissgerber 외 2015).
pub fn strip(rows: &[(String, f64, bool)], names: [&str; 2], title: &str) -> String {
    let (top, left, rr) = (26.0, 16.0, R + 8.0);
    let h = 200.0;
    let vals: Vec<f64> = rows.iter().map(|r| r.1).collect();
    let lo = (vals.iter().cloned().fold(0.0, f64::min) * 20.0).floor() / 20.0;
    let hi = (vals.iter().cloned().fold(0.0, f64::max) * 20.0).ceil() / 20.0;
    let x = |v: f64| left + (v - lo) / (hi - lo).max(0.01) * (W - left - rr);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let (yo, yh) = (top + 40.0, top + 110.0); // 회색 줄, 파랑 줄
    let step = [0.02, 0.05, 0.1, 0.2].into_iter().find(|st| (hi - lo) / st <= 6.0).unwrap_or(0.2);
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{top}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"14\">{}</text>",
                       x(v), x(v), h - 28.0, if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, x(v), h - 10.0, if v > 0.0 { format!("+{}", pct(v)) } else { pct(v) });
        v += step;
    }
    for (grp, y, c, name) in [(false, yo, GRAY, names[1]), (true, yh, BLUE, names[0])] {
        let g: Vec<f64> = rows.iter().filter(|r| r.2 == grp).map(|r| r.1).collect();
        let _ = write!(s, "<text x=\"{left}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\" font-weight=\"600\">{}</text>", y - 22.0, name.replace('&', "&amp;"));
        for r in rows.iter().filter(|r| r.2 == grp) {
            let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{y:.1}\" r=\"6\" fill=\"{c}\" fill-opacity=\"0.75\" stroke=\"#fff\" stroke-width=\"1.5\"><title>{}</title></circle>", x(r.1), r.0);
        }
        if let Some(m) = crate::stats::median(&g) {
            let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{c}\" stroke-width=\"2.5\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">median {}{}</text>",
                           x(m), x(m), y - 14.0, y + 14.0, x(m) + 5.0, y + 28.0, if m > 0.0 { "+" } else { "" }, pct(m));
        }
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod strip_test {
    use super::*;

    #[test]
    fn groups_and_medians() {
        let r = vec![("VA".into(), 0.12, true), ("PA".into(), 0.12, true), ("TX".into(), 0.03, false), ("CA".into(), -0.04, false)];
        let svg = strip(&r, ["PJM", "Other"], "t");
        assert_eq!(svg.matches("<circle").count(), 4);
        assert!(svg.contains("median +12%") && svg.contains("PJM"));
    }
}

#[cfg(test)]
mod multiples_test {
    use super::*;

    #[test]
    fn one_panel_per_case() {
        let c = vec![("2000".to_string(), vec![(0.0, 0.0), (100.0, -0.8), (200.0, -0.5)]), ("Now".to_string(), vec![(0.0, 0.0), (50.0, -0.4)])];
        let svg = multiples(&c, 200.0, "t");
        assert_eq!(svg.matches("<polyline").count(), 2);
        assert!(svg.contains("-80%") && svg.contains(BLUE));
    }
}

#[cfg(test)]
mod gauge_test {
    use super::*;

    #[test]
    fn needle_points_right_for_high_rank() {
        let svg = gauge(0.9, "Now", "Low", "High", "t");
        assert!(svg.contains("Now") && svg.matches("<path").count() == 3);
        // 바늘 끝 x가 가운데보다 오른쪽
        let i = svg.find("x2=\"").unwrap() + 4;
        let x: f64 = svg[i..].split('"').next().unwrap().parse().unwrap();
        assert!(x > W / 2.0);
    }
}

#[cfg(test)]
mod timeline_test {
    use super::*;

    #[test]
    fn one_dot_per_event_with_tone_color() {
        let it = vec![("2014".to_string(), "Mt. Gox collapses".to_string(), "Repayments start in 2024".to_string(), "bad".to_string()),
                      ("2019".to_string(), "Binance hot wallet hack".to_string(), "Covered in full".to_string(), "good".to_string())];
        let svg = timeline(&it, "t");
        assert_eq!(svg.matches("<circle").count(), 2);
        assert!(svg.contains("#eb6834") && svg.contains("#1baf7a") && svg.contains("Mt. Gox"));
    }
}

#[cfg(test)]
mod pictogram_test {
    use super::*;

    #[test]
    fn fills_the_right_number_of_units() {
        let svg = pictogram(&[("Fund".into(), 17, 100, "$17".into()), ("Portfolio".into(), 83, 100, "$83".into())], "t");
        assert_eq!(svg.matches(BLUE).count(), 17);
        assert_eq!(svg.matches("#eb6834").count(), 83);
        assert!(svg.contains("$83") && svg.contains("Portfolio"));
    }
}

#[cfg(test)]
mod cases_test {
    use super::*;

    #[test]
    fn one_row_per_case() {
        let r = vec![("1974".to_string(), -0.12, 0.2, Some(0.3)), ("1986".to_string(), -0.02, 0.1, Some(-0.12))];
        let svg = cases(&r, 0.07, ["Worst dip", "6 months", "1 year"], "t");
        assert_eq!(svg.matches("<circle").count(), 3 + 6);
        assert!(svg.contains("1986") && svg.contains("stroke-dasharray"));
    }
}

#[cfg(test)]
mod more_frames {
    use super::*;
    use crate::series::parse;

    #[test]
    fn band_and_dots() {
        let p = parse("d,v\n2000-01-01,1\n2001-01-01,3\n2002-01-01,2\n");
        assert!(band(&p, "t", false, "%").contains("<circle"));
        let d = dots(&[(0, -0.15, false), (400, -0.3, true)], 0.2, "1971", "2026", "t");
        assert!(d.matches("<circle").count() == 2 && d.contains("#2563eb"));
    }
}

/// 반응 경로 그림: 사건(선행 지표가 움직이기 시작한 날)을 0일로 맞추고 후행 지표의 변화 경로를 그룹마다 중앙값 선 + 사분위 띠로.
/// groups: (이름, 색, 사례마다 offsets 길이의 경로). 같은 축에서 오를 때·내릴 때의 모양(빠르기)을 비교한다(B5: 같은 축 위치).
pub fn response(groups: &[(String, &str, Vec<Vec<f64>>)], offsets: &[i64], title: &str) -> String {
    let q = |v: &mut Vec<f64>, f: f64| -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[((v.len() - 1) as f64 * f).round() as usize] };
    // offsets마다 (25%, 50%, 75%)
    let stats: Vec<Vec<(f64, f64, f64)>> = groups.iter().map(|(_, _, paths)| {
        (0..offsets.len()).map(|k| {
            let mut col: Vec<f64> = paths.iter().filter_map(|p| p.get(k).copied()).filter(|x| x.is_finite()).collect();
            if col.is_empty() { (0.0, 0.0, 0.0) } else { (q(&mut col, 0.25), q(&mut col, 0.5), q(&mut col, 0.75)) }
        }).collect()
    }).collect();
    let all: Vec<f64> = stats.iter().flatten().flat_map(|t| [t.0, t.2]).collect();
    let (mut lo, mut hi) = (all.iter().cloned().fold(0.0, f64::min), all.iter().cloned().fold(0.0, f64::max));
    lo = (lo * 20.0).floor() / 20.0;
    hi = (hi * 20.0).ceil() / 20.0;
    let xmax = *offsets.last().unwrap() as f64;
    let x = |d: f64| L + d / xmax * (W - L - R - 84.0);
    let y = |v: f64| T + (hi - v) / (hi - lo).max(0.01) * (H - T - B);
    let mut s = head(title);
    let step = if hi - lo > 0.3 { 0.1 } else { 0.05 };
    let mut v = (lo / step).ceil() * step;
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\">{}</text>",
                       W - R - 84.0, y(v), y(v), if v.abs() < 1e-9 { INK } else { "#e5e7eb" }, L - 6.0, y(v) + 4.0, pct(v));
        v += step;
    }
    for (g, (name, color, _)) in groups.iter().enumerate() {
        let st = &stats[g];
        let top: Vec<String> = offsets.iter().zip(st).map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.2))).collect();
        let bot: Vec<String> = offsets.iter().zip(st).rev().map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.0))).collect();
        let _ = write!(s, "<polygon points=\"{} {}\" fill=\"{color}\" fill-opacity=\"0.12\"/>", top.join(" "), bot.join(" "));
        let mid: Vec<String> = offsets.iter().zip(st).map(|(d, t)| format!("{:.1},{:.1}", x(*d as f64), y(t.1))).collect();
        let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2.5\"/>", mid.join(" "));
        let last = st.last().unwrap().1;
        let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" fill=\"{color}\" font-weight=\"600\">{}</text><text x=\"{}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}{}</text>",
                       W - R - 80.0, y(last) - 4.0, name.replace('&', "&amp;"), W - R - 80.0, y(last) + 13.0, if last > 0.0 { "+" } else { "" }, pct(last));
    }
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\">0</text><text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\">{} weeks</text></svg>",
                   H - 6.0, x(xmax), H - 6.0, offsets.last().unwrap() / 7);
    s
}

#[cfg(test)]
mod response_test {
    use super::*;

    #[test]
    fn response_draws_band_and_median() {
        let g = vec![("Drops".to_string(), BLUE, vec![vec![0.0, -0.05, -0.08], vec![0.0, -0.03, -0.06], vec![0.0, -0.04, -0.07]])];
        let svg = response(&g, &[0, 7, 14], "t");
        assert!(svg.contains("<polygon") && svg.contains("<polyline") && svg.contains("Drops") && svg.contains("2 weeks"));
    }
}

/// 구간 그림: 선 하나 + 기준선, 기준 이상으로 이어진 구간(spells)을 옅은 띠로 칠하고 띠 위에 길이(몇 년)를 쓴다.
/// "이런 고비가 과거엔 얼마나 갔나"를 한눈에(진행 중인 구간은 파랑 띠 + 'so far'). 짧은 구간(min_days 미만)은 칠하지 않는다.
pub fn spells(pts: &[Point], th: f64, min_days: i64, now: Option<(f64, &str)>, title: &str) -> String {
    let lo = pts.iter().map(|p| p.value).fold(f64::MAX, f64::min).min(th);
    let hi = pts.iter().map(|p| p.value).fold(f64::MIN, f64::max).max(th);
    let (lo, hi) = ((lo * 20.0).floor() / 20.0, (hi * 20.0).ceil() / 20.0);
    let top = T + 22.0; // 띠 위 글자 자리
    let (x0, x1) = (pts[0].day as f64, pts.last().unwrap().day as f64);
    let x = |d: f64| L + (d - x0) / (x1 - x0).max(1.0) * (W - L - R - 34.0);
    let y = |v: f64| top + (hi - v) / (hi - lo).max(1e-9) * (H - top - B);
    let mut s = head(title);
    // 띠 먼저(선 아래)
    let mut i = 0;
    while i < pts.len() {
        if pts[i].value < th { i += 1; continue; }
        let st = i;
        // 1년 안에 다시 넘으면 한 고비로 본다(1986~87년 반년 쉼에 글자 둘이 겹쳤다)
        loop {
            while i < pts.len() && pts[i].value >= th { i += 1; }
            let next = pts[i..].iter().position(|p| p.value >= th).map(|k| i + k);
            match next { Some(k) if pts[k].day - pts[i - 1].day <= 366 => i = k, _ => break }
        }
        let (a, b) = (&pts[st], &pts[i - 1]);
        let ongoing = i == pts.len();
        if b.day - a.day < min_days && !ongoing { continue; }
        let (xa, xb) = (x(a.day as f64), x(b.day as f64).max(x(a.day as f64) + 3.0));
        let fill = if ongoing { "#dbeafe" } else { SHADE };
        let _ = write!(s, "<rect x=\"{xa:.1}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{fill}\"/>", xb - xa, H - top - B);
        let yrs = (b.day - a.day) as f64 / 365.25;
        let lab = if ongoing { format!("{yrs:.1} yrs so far") } else { format!("{yrs:.1} yrs") };
        let anchor = if ongoing { "end" } else { "middle" };
        let tx = if ongoing { xb.max(xa + 10.0) + 30.0 } else { (xa + xb) / 2.0 };
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{}\" font-size=\"13\">{lab}</text>", top - 6.0, if ongoing { BLUE } else { INK });
    }
    // 눈금(5%p마다, 최대 6개)
    let mut v = lo;
    let step = if (hi - lo) / 0.05 > 6.0 { 0.1 } else { 0.05 };
    while v <= hi + 1e-9 {
        let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text>", W - R - 34.0, y(v), y(v), L - 6.0, y(v) + 4.0, pct(v));
        v += step;
    }
    let _ = write!(s, "<line x1=\"{L}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"{INK}\" stroke-dasharray=\"4 3\"/>", W - R - 34.0, y(th), y(th));
    let d: Vec<String> = pts.iter().map(|p| format!("{:.1},{:.1}", x(p.day as f64), y(p.value))).collect();
    let _ = write!(s, "<polyline points=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"2\"/>", d.join(" "));
    // 기준선 값 글자(눈금이 기준을 건너뛰면 선의 뜻이 안 보였다)
    let _ = write!(s, "<text x=\"{}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\" font-weight=\"600\">{}</text>", L - 6.0, y(th) + 4.0, pct(th));
    let l = pts.last().unwrap();
    let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{INK}\"/>", x(l.day as f64), y(l.value));
    // --now: 마지막 자료 뒤 오늘 추정(예: 지난 분기 집값에 이번 주 금리). 속 빈 파랑 점과 값, 마지막 값은 회색 글자로 아래에
    match now {
        Some((v, lab)) => {
            let xn = x(l.day as f64) + 10.0;
            let _ = write!(s, "<circle cx=\"{xn:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"#fff\" stroke=\"{BLUE}\" stroke-width=\"2.5\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-weight=\"600\" font-size=\"14\">{}</text><title>{lab}</title>", y(v), xn + 7.0, y(v) + 5.0, pct(v));
            let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"12\">{}</text>", xn + 7.0, y(l.value) + 16.0, pct(l.value));
        }
        None => { let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{BLUE}\" font-weight=\"600\" font-size=\"14\">{}</text>", x(l.day as f64) + 7.0, y(l.value) + 5.0, pct(l.value)); }
    }
    let _ = write!(s, "<text x=\"{L}\" y=\"{}\" fill=\"{INK}\" font-size=\"13\">{}</text><text x=\"{:.1}\" y=\"{}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"13\">{}</text></svg>", H - 6.0, &pts[0].date[..4], W - R - 34.0, H - 6.0, &l.date[..4]);
    s
}

/// 시차 그림: 신호마다 한 줄, 0(신호 시작)에서 사건 시작까지 막대와 몇 달 글자. 사건이 아직 없는 줄은 파랑 'so far'.
/// "보통 얼마 뒤에 왔나"와 "지금은 그 범위를 넘었나"를 같은 가로축 위치로(B5).
/// mark: 사건까지 mark개월 이하였던 막대를 주황으로(예: 1년 안에 침체가 온 신호), axis: 눈금 아래 축 이름
pub fn lead(rows: &[(String, f64, bool)], zero: &str, mark: f64, axis: &str, title: &str) -> String {
    let row = 26.0;
    let h = T + rows.len() as f64 * row + B + 8.0 + if axis.is_empty() { 0.0 } else { 20.0 };
    let max = rows.iter().map(|r| r.1).fold(12.0, f64::max);
    let step = if max > 72.0 { 24.0 } else if max > 36.0 { 12.0 } else { 6.0 };
    let xmax = (max / step).ceil() * step;
    let (l, r) = (52.0, 96.0);
    let x = |m: f64| l + m / xmax * (W - l - r);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    let ax = if axis.is_empty() { 0.0 } else { 20.0 };
    if !axis.is_empty() {
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>", (l + W - r) / 2.0, h - B + 16.0, axis.replace('&', "&amp;"));
    }
    let mut v = 0.0;
    while v <= xmax + 1e-9 {
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{T}\" y2=\"{:.1}\" stroke=\"#eef2f7\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"{INK}\" font-size=\"13\">{}</text>",
                       x(v), x(v), h - B - ax, x(v), h - B - ax + 16.0, match v { 0.0 => "0".into(), _ if step < 12.0 => format!("{v} mo"), 12.0 => "1 yr".into(), _ => format!("{} yrs", v / 12.0) });
        v += step;
    }
    for (k, (year, m, done)) in rows.iter().enumerate() {
        let y = T + k as f64 * row + 6.0;
        let color = if !*done { BLUE } else if *m <= mark { "#ea580c" } else { GRAY };
        let lab = match (*done, m.round()) { (false, _) => format!("{:.0} mo so far", m), (true, z) if z == 0.0 => zero.to_string(), _ => format!("{:.0} mo", m) };
        let _ = write!(s, "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"{INK}\" font-size=\"14\">{year}</text><rect x=\"{l}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" rx=\"2\" fill=\"{color}\"/><text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"13\"{}>{lab}</text>",
                       l - 8.0, y + 11.0, (x(*m) - l).max(2.0), x(*m) + 6.0, y + 11.0, if *done { INK } else { BLUE }, if *done { "" } else { " font-weight=\"600\"" });
    }
    s.push_str("</svg>");
    s
}

/// 신호판: 신호마다 한 줄 트랙(0~축 끝), 경고선 눈금과 '지금' 점. 경고선을 넘으면 점이 주황.
/// 서로 다른 단위의 신호를 '경고선까지 얼마나 남았나'라는 같은 위치로 읽게(B5). 값은 엔진 결과를 넣는다.
pub fn board(rows: &[(String, f64, f64, f64, String)], title: &str) -> String {
    let row = 58.0;
    let h = T + rows.len() as f64 * row + 10.0;
    let (l, r) = (16.0, 16.0);
    let mut s = head(title).replacen(&format!("0 0 {W} {H}\" width=\"{W}\" height=\"{H}\""), &format!("0 0 {W} {h}\" width=\"{W}\" height=\"{h}\""), 1);
    for (k, (name, now, trig, max, lab)) in rows.iter().enumerate() {
        let y = T + k as f64 * row + 18.0;
        let x = |v: f64| l + (v / max).clamp(0.0, 1.0) * (W - l - r);
        let over = now >= trig;
        let c = if over { "#ea580c" } else { BLUE };
        let _ = write!(s, "<text x=\"{l}\" y=\"{:.1}\" fill=\"{INK}\" font-size=\"14\">{}</text>", y, name.replace('&', "&amp;"));
        let _ = write!(s, "<rect x=\"{l}\" y=\"{:.1}\" width=\"{:.1}\" height=\"6\" rx=\"3\" fill=\"{SHADE}\"/><rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"6\" fill=\"#fed7aa\"/>",
                       y + 12.0, W - l - r, x(*trig), y + 12.0, W - r - x(*trig));
        let _ = write!(s, "<line x1=\"{:.1}\" x2=\"{:.1}\" y1=\"{:.1}\" y2=\"{:.1}\" stroke=\"#ea580c\" stroke-width=\"2\"/>", x(*trig), x(*trig), y + 6.0, y + 24.0);
        let _ = write!(s, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"7\" fill=\"{c}\" stroke=\"#fff\" stroke-width=\"2\"/>", x(*now), y + 15.0);
        let anchor = if x(*now) > W / 2.0 { "end" } else { "start" };
        let tx = if anchor == "end" { x(*now) - 10.0 } else { x(*now) + 10.0 };
        let _ = write!(s, "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"{c}\" font-size=\"13\" font-weight=\"600\">{}</text>", y + 38.0, lab.replace('&', "&amp;"));
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod lead_test {
    #[test]
    fn lead_marks_short_leads_orange() {
        let svg = super::lead(&[("1990".into(), 8.0, true), ("2000".into(), 30.0, true)], "0 mo", 12.0, "Months", "t");
        assert_eq!(svg.matches("#ea580c").count(), 1);
        assert!(svg.contains(">Months<"));
    }
}

#[cfg(test)]
mod board_test {
    #[test]
    fn board_marks_over_trigger_orange() {
        let svg = super::board(&[("A".into(), 0.6, 0.5, 1.0, "over".into()), ("B".into(), 0.1, 0.5, 1.0, "calm".into())], "t");
        assert!(svg.contains("#ea580c\" stroke=\"#fff") && svg.contains("calm"));
    }
}
