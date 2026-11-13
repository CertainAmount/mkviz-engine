//! 채권 펀드 손익분기: 금리가 jump만큼 오르면 값이 바로 약 듀레이션 × jump만큼 떨어지고(볼록성 반영),
//! 그 뒤로는 더 높은 수익률로 불어난다. 금리가 그대로였을 경로와 만나는 해가 손익분기다(대략 듀레이션 근처).
//! 단순화: 펀드 수익률은 시장 금리 변화를 바로 따르고, 이후 금리는 그대로라고 본다.

/// 금리 상승 직후 값의 변화(비율). 수정 듀레이션 d, 볼록성 c(없으면 d²/2 근사 대신 0), 상승폭 dy(0.01 = 1%p)
pub fn jump_loss(d: f64, c: f64, dy: f64) -> f64 {
    -d * dy + 0.5 * c * dy * dy
}

/// t년 뒤 두 경로의 값(시작 1): (금리 그대로, 금리 상승). y0는 연 수익률(0.045 = 4.5%)
pub fn values(d: f64, c: f64, y0: f64, dy: f64, t: f64) -> (f64, f64) {
    ((1.0 + y0).powf(t), (1.0 + jump_loss(d, c, dy)) * (1.0 + y0 + dy).powf(t))
}

/// 손익분기(년): 상승 경로가 그대로 경로를 따라잡는 때
pub fn breakeven(d: f64, c: f64, y0: f64, dy: f64) -> Option<f64> {
    let k = 1.0 + jump_loss(d, c, dy);
    if dy <= 0.0 || k <= 0.0 { return None; }
    Some(-(k.ln()) / ((1.0 + y0 + dy).ln() - (1.0 + y0).ln()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakeven_near_duration() {
        // 듀레이션 6년, 4.5% → 5.5%: 바로 약 6% 손실, 손익분기는 약 6년
        let l = jump_loss(6.0, 0.0, 0.01);
        assert!((l + 0.06).abs() < 1e-9);
        let b = breakeven(6.0, 0.0, 0.045, 0.01).unwrap();
        assert!(b > 5.5 && b < 7.0, "{b}");
        let (a, j) = values(6.0, 0.0, 0.045, 0.01, b);
        assert!((a - j).abs() < 1e-9);
    }
}
