use crate::ai::ai_consts::*;

fn node_layer1_gemm(a: &[f64; 42], b: &[[f64; 42]; 64], c: &[f64; 64], y: &mut [f64; 64]) {
    let m: usize = 64;
    let n: usize = 42;
    let alpha: f64 = 1.0000000000000000000;
    let beta: f64 = 1.0000000000000000000;
    for j in 0..m {
        let mut ab_rc = 0.0;
        for i in 0..n {
            ab_rc += a[i] * b[j][i];
        }
        y[j] = ab_rc * alpha + c[j] * beta;
    }
}

fn node_relu_relu(x: &[f64; 64], y: &mut [f64; 64]) {
    for i in 0..64 {
        y[i] = if x[i] > 0.0 { x[i] } else { 0.0 };
    }
}

fn node_layer2_gemm(a: &[f64; 64], b: &[[f64; 64]; 32], c: &[f64; 32], y: &mut [f64; 32]) {
    let m: usize = 64;
    let n: usize = 32;
    let alpha = 1.0000000000000000000;
    let beta = 1.0000000000000000000;
    for j in 0..n {
        let mut ab_rc = 0.0_f64;
        for i in 0..m {
            ab_rc += a[i] * b[j][i];
        }
        y[j] = ab_rc * alpha + c[j] * beta;
    }
}

fn node_relu_1_relu(x: &[f64; 32], y: &mut [f64; 32]) {
    for i in 0..32_usize {
        y[i] = if x[i] > 0.0 { x[i] } else { 0.0 }
    }
}

fn node_layer3_gemm(a: &[f64; 32], b: &[[f64; 32]; 16], c: &[f64; 16], y: &mut [f64; 16]) {
    let m: usize = 32;
    let n: usize = 16;
    let alpha = 1.0000000000000000000;
    let beta = 1.0000000000000000000;
    for j in 0..n {
        let mut ab_rc = 0.0_f64;
        for i in 0..m {
            ab_rc += a[i] * b[j][i];
        }
        y[j] = ab_rc * alpha + c[j] * beta;
    }
}

fn node_relu_2_relu(x: &[f64; 16], y: &mut [f64; 16]) {
    for i in 0..16_usize {
        y[i] = if x[i] > 0.0 { x[i] } else { 0.0 }
    }
}

fn node_layer4_gemm(a: &[f64; 16], b: &[[f64; 16]; 1], c: &[f64; 1], y: &mut [f64; 1]) {
    let m: usize = 16;
    let n: usize = 1;
    let alpha = 1.0000000000000000000;
    let beta = 1.0000000000000000000;
    for j in 0..n {
        let mut ab_rc = 0.0_f64;
        for i in 0..m {
            ab_rc += a[i] * b[j][i];
        }
        y[j] = ab_rc * alpha + c[j] * beta;
    }
}

fn node_sigmoid(x: &[f64; 1], y: &mut [f64; 1]) {
    y[0] = 1.0 / (1.0 + (-x[0]).exp());
}

pub fn entry(tensor_input: [f64; 42], tensor_output: &mut [f64; 1]) {
    let mut tensor_layer1_gemm_output_0: [f64; 64] = [0.0; 64];
    let mut tensor_layer2_gemm_output_0: [f64; 32] = [0.0; 32];
    let mut tensor_layer3_gemm_output_0: [f64; 16] = [0.0; 16];
    let mut tensor_layer4_gemm_output_0: [f64; 1] = [0.0];
    let mut tensor_relu_relu_output_0: [f64; 64] = [0.0; 64];
    let mut tensor_relu_1_relu_output_0: [f64; 32] = [0.0; 32];
    let mut tensor_relu_2_relu_output_0: [f64; 16] = [0.0; 16];

    node_layer1_gemm(
        &tensor_input,
        &TENSOR_LAYER1_WEIGHT,
        &TENSOR_LAYER1_BIAS,
        &mut tensor_layer1_gemm_output_0,
    );
    node_relu_relu(&tensor_layer1_gemm_output_0, &mut tensor_relu_relu_output_0);
    node_layer2_gemm(
        &tensor_relu_relu_output_0,
        &TENSOR_LAYER2_WEIGHT,
        &TENSOR_LAYER2_BIAS,
        &mut tensor_layer2_gemm_output_0,
    );
    node_relu_1_relu(
        &tensor_layer2_gemm_output_0,
        &mut tensor_relu_1_relu_output_0,
    );
    node_layer3_gemm(
        &tensor_relu_1_relu_output_0,
        &TENSOR_LAYER3_WEIGHT,
        &TENSOR_LAYER3_BIAS,
        &mut tensor_layer3_gemm_output_0,
    );
    node_relu_2_relu(
        &tensor_layer3_gemm_output_0,
        &mut tensor_relu_2_relu_output_0,
    );
    node_layer4_gemm(
        &tensor_relu_2_relu_output_0,
        &TENSOR_LAYER4_WEIGHT,
        &TENSOR_LAYER4_BIAS,
        &mut tensor_layer4_gemm_output_0,
    );
    node_sigmoid(&tensor_layer4_gemm_output_0, tensor_output);
}
