use super::ai_model::entry;
use crate::state::AIOriginalDataAccount;

const SCORE_WEIGHTS: [f64; 42] = [
    8.0, 0.0, 0.0, 2.0, 4.0, -2.0, 1.0, -0.5, 2.0, -1.0, 2.0, 1.0, -0.5, -1.0, -2.0, -2.0, 1.0,
    -1.0, 0.5, 1.0, -1.0, 0.5, 4.0, 2.0, 1.0, 1.0, 1.0, 4.0, 4.0, 2.0, 0.0, 2.0, 0.0, 4.0, 2.0,
    1.0, 4.0, 2.0, -1.0, 2.0, 2.0, 2.0,
];

pub fn inference(input: &AIOriginalDataAccount) -> bool {
    let datas: [f64; 42] = input.datas;
    let mut result: [f64; 1] = [0.0];
    entry(datas, &mut result);
    let score = calculate_score(&datas);
    let regulated = determine_leverage(score, result[0], input.sol);
    regulated
}

fn calculate_score(datas: &[f64; 42]) -> f64 {
    let mut score = 0.0;
    for i in 0..=41 {
        score += datas[i] * SCORE_WEIGHTS[i];
    }
    score
}

fn determine_leverage(score: f64, probability: f64, sol_amount: f64) -> bool {
    const MIN_SOL: f64 = 4.0;
    const MID_SOL: f64 = 16.0;
    const EPSILON: f64 = 0.0;
    if sol_amount < MIN_SOL {
        return false;
    }

    let normalized_score = score / 31.5;

    if sol_amount > MID_SOL {
        if normalized_score < 0.2 && probability < 0.1 {
            return false;
        }
        return true;
    }

    let progress = round((sol_amount - MIN_SOL) / (MID_SOL - MIN_SOL));
    let score_requirement = round(0.82 - 0.2 * progress);
    let prob_requirement = round(0.7 - 0.3 * progress);

    if normalized_score + EPSILON >= score_requirement && probability + EPSILON >= prob_requirement{
        return true;
    }

    if probability >=0.9{
        let prob_excess =  0.1_f64.min(probability - 0.9);
        let min_score_ratio = 0.3 - (prob_excess / 0.1) * 0.15;
        let min_score = score_requirement * min_score_ratio;
        if normalized_score >= min_score - EPSILON{
            return true;
        }
    }

    if (sol_amount - MID_SOL).abs() < EPSILON && normalized_score >=0.5 -EPSILON && probability >=0.1-EPSILON{
        return  true;
    }

    return false;
}

fn round(number: f64) -> f64 {
    (number * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use solana_program::msg;

    use crate::ai::ai_model::entry;

    use super::{calculate_score, determine_leverage, AIOriginalDataAccount};

    #[test]
    fn test_leverage() {

        let datas = [
            [45.536401911, 0.0, 5.536401911, 0.12727, 54.255472665, 49.224119897, 2.41878, 1.981246553, 154.0, 177.0, 4.3e-08, 13.160756974, 4.341674504, 18723500779.788143, 18723500779.788143, 18.7235, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 331.0, 0.0, 5.2e-08, 0.0],
        [45.010284089, 45.536401911, 5.010284089, 0.11653, 0.1992, 0.725317822, 0.1992, 0.303969999, 1.0, 4.0, 5.1e-08, 0.0, 0.0, 299072043.19297, 299072043.19297, 0.29907, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 331.0, 5.1e-08, 5.2e-08],
        [45.22240409, 45.010284089, 5.22240409, 0.12089, 0.37982, 0.167699999, 0.37982, 0.167699999, 1.0, 1.0, 5.1e-08, 0.0, 0.0, 112981893.99649, 112981893.99649, 0.11298, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 5.0, 5.1e-08, 5.1e-08],
        [44.673891705, 45.22240409, 4.673891705, 0.10952, 0.246969999, 0.795482384, 0.129759999, 0.305328018, 2.0, 5.0, 5.1e-08, 0.0, 0.0, 351796705.215704, 351796705.215704, 0.3518, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0, 2.0, 5e-08, 5.1e-08],
        [44.224358648, 44.673891705, 4.224358648, 0.09999, 4.706315617, 5.155848674, 0.8029, 2.124409999, 16.0, 15.0, 4.7e-08, 0.0, 2.124409999, 2484871048.879124, 2484871048.879124, 2.48487, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 31.0, 7.0, 4.9e-08, 5e-08],
        [44.134648649, 44.224358648, 4.134648649, 0.09807, 0.2601, 0.349809999, 0.2601, 0.349809999, 1.0, 1.0, 4.8e-08, 0.0, 0.0, 156637295.093712, 156637295.093712, 0.15664, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 31.0, 4.9e-08, 4.9e-08],
        [44.009359798, 45.301469796, 4.009359798, 0.09537, 0.01069, 1.302799998, 0.01069, 1.282429999, 1.0, 2.0, 5e-08, 0.0, 1.282429999, 231633473.19891, 231633473.19891, 0.23163, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 53.0, 4.8e-08, 5.1e-08],
        [44.000000001, 0.0, 4.000000001, 0.09516, 4.355, 0.354999999, 2.0, 0.354999999, 3.0, 1.0, 4.3e-08, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 4.8e-08, 0.0]
        ];
        // let mut datas = [0.0;33];
        // datas[2] = 10.0;
        // let datas: [f64; 33] = [ 30.04688725, 30.04688725, 30.0, 1e-05, 0.046887248, 0.0, 50.42834283, 50.38145558, 5.828796534, 6.394745508, 49.0, 48.0, 5.1e-08, 36.65467794, 36.54587439, 0.242, 1440307.242, 5.45e-08, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.005286106, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 49.0, 49.0, ];
        // let datas: [f64; 33] = [89.940110266, 150.002, 49.940110266, 0.581250011920929, 0.1, 59.560270837, 0.1, 59.560270837, 1.0, 1.0, 3.33E-7, 0.0, 59.560270837, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.02E-7, 5.63E-7];
        // let datas = [150.0, 140.0, 10.0, 0.0, 10.0, 0.0, 1.0, 0.0, 5.25E-7, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.62E-7, 0.0, 4.9E-7, 110.0, 0.7676600217819214];
        // let datas = [44.382795298, 45.308335296, 4.382795298, 0.10337, 0.094760000, 1.020299998, 0.094760000, 0.637799999, 1.0, 2.0, 0.000000050, 0.000000000, 0.000000000, 117562041.845223, 117562041.845223, 0.11756, 0.000000000, 0.000000000, 0.0, 0.000000000, 0.000000000, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 216.0, 0.000000049, 0.000000051];
        // 模拟创建一个空的 OriginalDataAccount
        for data in datas {
            test_leverage_with_datas(data)
        }
       
    }

    fn test_leverage_with_datas(datas: [f64; 33]) {
        // 模拟创建一个空的 OriginalDataAccount
        let mut original_account: AIOriginalDataAccount = AIOriginalDataAccount {
            bump: 2_u8,
            period: 1,
            period_done: false,
            regulated: false,
            sol: 0.0_f64,
            datas: [0.0; 42],
        };
        original_account.init();
        let _ = original_account.feed(&datas);
        let mut result: [f64; 1] = [0.0];
        entry(original_account.datas, &mut result);
        let score = calculate_score(&original_account.datas);
        let regulated = determine_leverage(score, result[0], original_account.sol);
        msg!(
            "probability:{}, score:{}, regulated:{}",
            result[0],
            score,
            regulated
        );
    }

    #[test]
    fn test_determine_leverage() {
        let test_cases = vec![
        // 1. SOL < 4 测试
        TestCase { sol: 2.0, score: 28.0, launch_prob: 0.95, expected: false, desc: "SOL<4: 高评分高概率".to_string() },
        TestCase { sol: 3.9, score: 25.0, launch_prob: 0.85, expected: false, desc: "SOL<4: 接近边界".to_string() },

        // 2. SOL = 4 边界测试
        TestCase { sol: 4.0, score: 22.0, launch_prob: 0.8, expected: true, desc: "SOL=4: 达到基础要求".to_string() },
        TestCase { sol: 4.0, score: 20.0, launch_prob: 0.75, expected: false, desc: "SOL=4: 概率略低".to_string() },
        TestCase { sol: 4.0, score: 28.0, launch_prob: 0.65, expected: true, desc: "SOL=4: 高评分补偿".to_string() },
        TestCase { sol: 4.0, score: 18.0, launch_prob: 0.95, expected: true, desc: "SOL=4: 高概率补偿".to_string() },

        // 3. SOL = 10 中间测试
        TestCase { sol: 10.0, score: 19.0, launch_prob: 0.7, expected: true, desc: "SOL=10: 达到要求".to_string() },
        TestCase { sol: 10.0, score: 16.0, launch_prob: 0.65, expected: false, desc: "SOL=10: 均不足".to_string() },
        TestCase { sol: 10.0, score: 25.0, launch_prob: 0.55, expected: true, desc: "SOL=10: 高评分补偿".to_string() },
        TestCase { sol: 10.0, score: 5.0, launch_prob: 0.95, expected: true, desc: "SOL=10: 极高概率补偿".to_string() },

        // 4. SOL = 16 边界测试
        TestCase { sol: 16.0, score: 16.0, launch_prob: 0.6, expected: true, desc: "SOL=16: 达到要求".to_string() },
        TestCase { sol: 16.0, score: 12.0, launch_prob: 0.55, expected: false, desc: "SOL=16: 均不足".to_string() },
        TestCase { sol: 16.0, score: 25.0, launch_prob: 0.45, expected: true, desc: "SOL=16: 高评分补偿".to_string() },
        TestCase { sol: 16.0, score: 14.0, launch_prob: 0.9, expected: true, desc: "SOL=16: 高概率补偿".to_string() },

        // 5. SOL > 16 测试（99.99%加杠杆情况）
        TestCase { sol: 20.0, score: 5.0, launch_prob: 0.25, expected: true, desc: "SOL>16: 低分低概率".to_string() },
        TestCase { sol: 25.0, score: 8.0, launch_prob: 0.3, expected: true, desc: "SOL>16: 低分中概率".to_string() },
        TestCase { sol: 30.0, score: 3.0, launch_prob: 0.15, expected: false, desc: "SOL>16: 极低分极低概率".to_string() },
        TestCase { sol: 50.0, score: 2.0, launch_prob: 0.18, expected: false, desc: "SOL>16: 极端低分低概率".to_string() },
        TestCase { sol: 100.0, score: 15.0, launch_prob: 0.35, expected: true, desc: "SOL>16: 中等表现".to_string() },
        TestCase { sol: 200.0, score: 6.0, launch_prob: 0.22, expected: true, desc: "SOL>16: 大额低表现".to_string() },
        ];

        let mut matches: i32 = 0;
        let total = test_cases.len();
        for case in test_cases {
            let decision = determine_leverage(case.score, case.launch_prob, case.sol);
            let match_status = if decision==case.expected { "✓" } else { "✗" };
            if match_status.eq_ignore_ascii_case("✓") {
                matches += 1;
            }

            msg!("{:<8.2} {:<8.1} {:<8.2} {:<8} {:<8} {:<4} {}", 
                case.sol, case.score, case.launch_prob, 
                decision, case.expected, 
                match_status, case.desc);
        }

        msg!("准确率: {}/{} ({:.1}%)", matches, total, (matches as f64 / total as f64) * 100.0);
    }

#[derive(Debug)]
struct TestCase {
    sol: f64,
    score: f64,
    launch_prob: f64,
    expected: bool,
    desc: String,
}
}
